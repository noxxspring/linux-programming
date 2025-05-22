use std::{collections::VecDeque, sync::{Arc, Mutex}, thread, time::Duration};

#[derive(Clone, Debug, PartialEq)]
enum Policy {
    CFS,
    FIFO,
    RR,
}

#[derive(Clone,Debug, PartialEq)]
enum Taskstate {
    Ready,
    Running,
    Sleeping(u32),  // sleeping for n scheduler ticks
    Finished,
    
}

#[derive(Clone, Debug)]
struct Task {
    name: String,
    policy: Policy,
    priority: u8,
    vruntime: u32,
    time_slice: u32,  // RR time slice
    total_runtime: u32, // Total CPU time consumed
    cpu_usage: u32,    //Accumulate CPU usage (simulated)
    nice: i8,         // Nice value (-20 to 19)
    state: Taskstate,    //Ready, Running, Sleeping
    deadlne: Option<u32>,  // Absolute deadline tick count (Optional)
}

struct Scheduler {
    rt_queue: VecDeque<Task>, // Realtime tasks FIFO + RR
    cfs_queue: Vec<Task>,      // CFS tasks
    tick: u32,                // Scheduler tick count
}

impl Scheduler {
    fn new() -> Self {
        Self {
            rt_queue: VecDeque::new(),
            cfs_queue: Vec::new(),
            tick: 0,
        }
    }

    fn add_task(&mut self, mut  task: Task) {
        task.state = Taskstate::Ready; // ensure task is ready
        match task.policy {
            Policy::CFS => self.cfs_queue.push(task),
            Policy::FIFO | Policy::RR => self.rt_queue.push_back(task),
            
        }
    }

    // convert nice to weight (Simplified)
   fn nice_to_weight(nice: i8) -> u32 {
    // Clamp nice value between -20 and 19
    let nice = nice.clamp(-20, 19);

    // Linux nice-to-weight table for nice from -20 to 19
    const WEIGHTS: [u32; 40] = [
        88761, 71755, 56483, 46273, 36291, 29154, 23254, 18705,
        14949, 11916, 9548, 7620, 6100, 4904, 3906, 3121,
        2501, 1991, 1586, 1277, 1024, 820, 655, 526,
        423, 335, 272, 215, 172, 137, 110, 87,
        70, 56, 45, 36, 29, 23, 18, 15,
    ];

    // Index into the weights array: nice + 20 to shift range -20..19 to 0..39
    WEIGHTS[(nice + 20) as usize]
}



    // wake sleeping tasks: decrease sleep time , move to ready when done
   fn wake_sleeping_tasks(&mut self) {
    for task in &mut self.rt_queue.iter_mut().chain(self.cfs_queue.iter_mut()) {
        if let Taskstate::Sleeping(ticks) = &mut task.state {
            if *ticks > 1 {
                *ticks -= 1;
            } else {
                task.state = Taskstate::Ready;
                println!("Task {} woke up!", task.name);
            }
        }
    }
}



    // schedule one task per tick
    fn schedule(&mut self) -> Option<Task> {
        println!("\n[Tick {}] RT tasks: {}, CFS tasks: {}", 
    self.tick, 
    self.rt_queue.len(), 
    self.cfs_queue.len());
    for task in self.rt_queue.iter().chain(self.cfs_queue.iter()) {
    println!("- {}: {:?}", task.name, task.state);
}
        self.tick += 1;
        self.wake_sleeping_tasks();

        // //remove sleeping tasks from consideration
        // self.rt_queue.retain(|t| t.state == Taskstate::Ready);
        // self.cfs_queue.retain(|t| t.state == Taskstate::Ready);


        // real time scheduling
        // priority 1: FIFO tasks (runs once, no requeue, marked finished)
        if let Some(pos)  = self
        .rt_queue
        .iter()
        .position(|t| t.policy == Policy::FIFO && t.state == Taskstate::Ready)
        {
            let mut task = self.rt_queue.remove(pos).unwrap();
            println!("-> [Real-Time FIFO] Running: {}", task.name);
            task.total_runtime += 10;
            task.cpu_usage += 10;

            // mark task Finihsed, so as to not schedule agan
            task.state = Taskstate::Finished;
            return Some(task);
        }

        //Priotity 2: RR task (run, possibly sleep, then requeue)
      // In schedule(), modify the RR section:
     if let Some(mut task) = self.rt_queue.pop_front() {
     if task.policy == Policy::RR && task.state == Taskstate::Ready {
        println!("-> [Real-Time RR] Running: {}", task.name);
        task.total_runtime += 10;
        task.cpu_usage += 10;
        
        // Only sleep if time slice expired AND not already sleeping
        if task.total_runtime % task.time_slice == 0 && !matches!(task.state, Taskstate::Sleeping(_)) {
            task.state = Taskstate::Sleeping(3);
        }
        self.rt_queue.push_back(task.clone());
        return Some(task);
    } else {
        // Requeue if not ready or not RR
        self.rt_queue.push_back(task);
    }
}

        // Completely fair Schedular(CFS)
        if !self.cfs_queue.is_empty() {
            //Sort by vruntime (lowest runs first)
            self.cfs_queue
            .sort_by_key(|t| t.vruntime);

        // Pick the task with the lowest vruntime
        let mut task = self.cfs_queue.remove(0);

        println!("-> [CFS] Running: {}", task.name);

        // Calculate weighted vruntime increment based on nice value
        let weight = Scheduler::nice_to_weight(task.nice);

        //increase vruntime scaled by weight 
        task.vruntime += 10 * (1024 / weight);
        task.total_runtime += 10;
        task.cpu_usage += 10;
        task.state =Taskstate::Running;

        //Mark task ready for next round
        task.state = Taskstate::Ready;

        // Requeue the task
        self.cfs_queue.push(task.clone());

        return Some(task);
        }

        // if no task to schedule
        println!("-> [Idle] No task to run");
        None
    }
}
fn main() {
    let scheduler = Arc::new(Mutex::new(Scheduler::new()));

    {
        let mut sched = scheduler.lock().unwrap();

        //Add CFS task, nice 10 (lower priority)
        sched.add_task(Task {
            name: "background-indexer-low".into(),
            policy: Policy::CFS,
            priority: 0,
            vruntime: 0,
            time_slice: 0,
            total_runtime: 0,
            cpu_usage: 0,
            nice: 10,
            state: Taskstate::Ready,
            deadlne: None,
        });

         //Add CFS task with higer priority
        sched.add_task(Task {
            name: "background-indexer-higer".into(),
            policy: Policy::CFS,
            priority: 0,
            vruntime: 0,
            time_slice: 0,
            total_runtime: 0,
            cpu_usage: 0,
            nice: -5,
            state: Taskstate::Ready,
            deadlne: None,
        });

        // Add RR task, nice 0, time_slice 30
        sched.add_task(Task {
            name: "video-player".into(),
            policy: Policy::RR,
            priority: 1,
            vruntime: 0,
            time_slice: 30,
            total_runtime: 0,
            cpu_usage: 0,
            nice: 0,
            state: Taskstate::Ready,
            deadlne: None,
        });

        // Add FIFO task
        sched.add_task(Task {
            name: "live-stream".into(),
            policy: Policy::FIFO,
            priority: 0,
            vruntime: 0,
            time_slice: 0,
            total_runtime: 0,
            cpu_usage: 0,
            nice: 0,
            state: Taskstate::Ready,
            deadlne:None
        });
    }

    // spawn a background thread to run scheduler ticks asynchronously
    let scheduler_clone = Arc::clone(&scheduler);
    thread::spawn(move || loop {
        {
            let mut sched = scheduler_clone.lock().unwrap();
            sched.schedule();
        }
        thread::sleep(Duration::from_millis(200)); // stimulate immer tick 200ms
    });

    // Main thread can stimulate external events, sleeping task etc.
    // Example: put "video-player" to sleep after 2 secs
    thread::sleep(Duration::from_secs(2));
    {
        let mut sched = scheduler.lock().unwrap();
        for task in sched.rt_queue.iter_mut() {
            if task.name == "video-player" {
                println!("putting video-player to sleep for 5 ticks");
                task.state = Taskstate::Sleeping(5);
            }
        }
    }

    // let scheduler run for some seconds to see effecs
    thread::sleep(Duration::from_secs(5));
}
