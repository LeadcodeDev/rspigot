use rspigot_domain::mth;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flag {
    Move,
    Look,
    Jump,
    Target,
}

impl Flag {
    pub const ALL: [Self; 4] = [Self::Move, Self::Look, Self::Jump, Self::Target];

    const fn index(self) -> usize {
        self as usize
    }

    const fn bit(self) -> u8 {
        1 << self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Flags(u8);

impl Flags {
    pub const NONE: Self = Self(0);

    pub const fn of(flag: Flag) -> Self {
        Self(flag.bit())
    }

    pub const fn with(self, flag: Flag) -> Self {
        Self(self.0 | flag.bit())
    }

    pub const fn contains(self, flag: Flag) -> bool {
        self.0 & flag.bit() != 0
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub fn insert(&mut self, flag: Flag) {
        self.0 |= flag.bit();
    }

    pub fn remove(&mut self, flag: Flag) {
        self.0 &= !flag.bit();
    }

    pub fn iter(self) -> impl Iterator<Item = Flag> {
        Flag::ALL
            .into_iter()
            .filter(move |flag| self.contains(*flag))
    }
}

pub const fn reduced_tick_delay(ticks: i32) -> i32 {
    mth::positive_ceil_div(ticks, 2)
}

pub const fn adjusted_tick_delay(every_tick: bool, ticks: i32) -> i32 {
    if every_tick {
        ticks
    } else {
        reduced_tick_delay(ticks)
    }
}

pub trait Goal<C> {
    fn flags(&self) -> Flags;

    fn can_use(&mut self, context: &mut C) -> bool;

    fn can_continue_to_use(&mut self, context: &mut C) -> bool {
        self.can_use(context)
    }

    fn is_interruptable(&self) -> bool {
        true
    }

    fn start(&mut self, _context: &mut C) {}

    fn stop(&mut self, _context: &mut C) {}

    fn requires_update_every_tick(&self) -> bool {
        false
    }

    fn tick(&mut self, _context: &mut C) {}
}

#[derive(Debug, Clone, PartialEq)]
pub struct WrappedGoal<G> {
    goal: G,
    priority: i32,
    running: bool,
}

impl<G> WrappedGoal<G> {
    pub fn goal(&self) -> &G {
        &self.goal
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    fn start<C>(&mut self, context: &mut C)
    where
        G: Goal<C>,
    {
        if !self.running {
            self.running = true;
            self.goal.start(context);
        }
    }

    fn stop<C>(&mut self, context: &mut C)
    where
        G: Goal<C>,
    {
        if self.running {
            self.running = false;
            self.goal.stop(context);
        }
    }
}

const NO_GOAL_PRIORITY: i32 = i32::MAX;

#[derive(Debug, Clone, PartialEq)]
pub struct GoalSelector<G> {
    goals: Vec<WrappedGoal<G>>,
    locked: [Option<usize>; 4],
    disabled: Flags,
}

impl<G> Default for GoalSelector<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G> GoalSelector<G> {
    pub fn new() -> Self {
        Self {
            goals: Vec::new(),
            locked: [None; 4],
            disabled: Flags::NONE,
        }
    }

    pub fn add_goal(&mut self, priority: i32, goal: G) {
        self.goals.push(WrappedGoal {
            goal,
            priority,
            running: false,
        });
    }

    pub fn goals(&self) -> &[WrappedGoal<G>] {
        &self.goals
    }

    pub fn goals_mut(&mut self) -> impl Iterator<Item = &mut G> {
        self.goals.iter_mut().map(|wrapped| &mut wrapped.goal)
    }

    pub fn remove_goals<C>(&mut self, context: &mut C, remove: impl Fn(&G) -> bool)
    where
        G: Goal<C>,
    {
        for wrapped in &mut self.goals {
            if remove(&wrapped.goal) {
                wrapped.stop(context);
            }
        }
        let mut mapping = Vec::with_capacity(self.goals.len());
        let mut kept = 0;
        for wrapped in &self.goals {
            if remove(&wrapped.goal) {
                mapping.push(None);
            } else {
                mapping.push(Some(kept));
                kept += 1;
            }
        }
        for slot in &mut self.locked {
            *slot = slot.and_then(|index| mapping[index]);
        }
        self.goals.retain(|wrapped| !remove(&wrapped.goal));
    }

    pub fn locked(&self, flag: Flag) -> Option<&WrappedGoal<G>> {
        self.locked[flag.index()].map(|index| &self.goals[index])
    }

    pub fn disable_control_flag(&mut self, flag: Flag) {
        self.disabled.insert(flag);
    }

    pub fn enable_control_flag(&mut self, flag: Flag) {
        self.disabled.remove(flag);
    }

    pub fn set_control_flag(&mut self, flag: Flag, enabled: bool) {
        if enabled {
            self.enable_control_flag(flag);
        } else {
            self.disable_control_flag(flag);
        }
    }

    fn replaceable_for_all_flags<C>(&self, index: usize) -> bool
    where
        G: Goal<C>,
    {
        let candidate = &self.goals[index];
        candidate.goal.flags().iter().all(|flag| {
            let (interruptable, priority) = match self.locked[flag.index()] {
                Some(holder) => {
                    let holder = &self.goals[holder];
                    (holder.goal.is_interruptable(), holder.priority)
                }
                None => (true, NO_GOAL_PRIORITY),
            };
            interruptable && candidate.priority < priority
        })
    }

    pub fn tick<C>(&mut self, context: &mut C)
    where
        G: Goal<C>,
    {
        let disabled = self.disabled;
        for wrapped in &mut self.goals {
            if wrapped.running
                && (wrapped.goal.flags().intersects(disabled)
                    || !wrapped.goal.can_continue_to_use(context))
            {
                wrapped.stop(context);
            }
        }
        for slot in &mut self.locked {
            if slot.is_some_and(|index| !self.goals[index].running) {
                *slot = None;
            }
        }
        for index in 0..self.goals.len() {
            let wrapped = &self.goals[index];
            if wrapped.running
                || wrapped.goal.flags().intersects(disabled)
                || !self.replaceable_for_all_flags::<C>(index)
                || !self.goals[index].goal.can_use(context)
            {
                continue;
            }
            for flag in self.goals[index].goal.flags().iter() {
                if let Some(holder) = self.locked[flag.index()] {
                    self.goals[holder].stop(context);
                }
                self.locked[flag.index()] = Some(index);
            }
            self.goals[index].start(context);
        }
        self.tick_running_goals(context, true);
    }

    pub fn tick_running_goals<C>(&mut self, context: &mut C, force: bool)
    where
        G: Goal<C>,
    {
        for wrapped in &mut self.goals {
            if wrapped.running && (force || wrapped.goal.requires_update_every_tick()) {
                wrapped.goal.tick(context);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Probe {
        name: &'static str,
        flags: Flags,
        usable: bool,
        interruptable: bool,
        every_tick: bool,
    }

    impl Probe {
        fn new(name: &'static str, flags: Flags) -> Self {
            Self {
                name,
                flags,
                usable: true,
                interruptable: true,
                every_tick: false,
            }
        }
    }

    #[derive(Default)]
    struct Log(Vec<String>);

    impl Goal<Log> for Probe {
        fn flags(&self) -> Flags {
            self.flags
        }

        fn can_use(&mut self, _context: &mut Log) -> bool {
            self.usable
        }

        fn is_interruptable(&self) -> bool {
            self.interruptable
        }

        fn start(&mut self, context: &mut Log) {
            context.0.push(format!("start {}", self.name));
        }

        fn stop(&mut self, context: &mut Log) {
            context.0.push(format!("stop {}", self.name));
        }

        fn requires_update_every_tick(&self) -> bool {
            self.every_tick
        }

        fn tick(&mut self, context: &mut Log) {
            context.0.push(format!("tick {}", self.name));
        }
    }

    fn running(selector: &GoalSelector<Probe>) -> Vec<&'static str> {
        selector
            .goals()
            .iter()
            .filter(|goal| goal.is_running())
            .map(|goal| goal.goal().name)
            .collect()
    }

    #[test]
    fn shared_flag_goes_to_the_better_priority() {
        let mut selector = GoalSelector::new();
        selector.add_goal(5, Probe::new("stroll", Flags::of(Flag::Move)));
        selector.add_goal(
            3,
            Probe::new("tempt", Flags::of(Flag::Move).with(Flag::Look)),
        );
        selector.add_goal(6, Probe::new("look", Flags::of(Flag::Look)));
        let mut log = Log::default();
        selector.tick(&mut log);
        assert_eq!(running(&selector), vec!["tempt"]);
        assert_eq!(
            log.0,
            vec!["start stroll", "stop stroll", "start tempt", "tick tempt"]
        );
        assert_eq!(
            selector.locked(Flag::Move).map(|goal| goal.goal().name),
            Some("tempt")
        );
        assert!(selector.locked(Flag::Jump).is_none());
    }

    #[test]
    fn worse_priority_cannot_replace_and_equal_priority_cannot_either() {
        let mut selector = GoalSelector::new();
        selector.add_goal(2, Probe::new("first", Flags::of(Flag::Move)));
        selector.add_goal(2, Probe::new("second", Flags::of(Flag::Move)));
        selector.add_goal(4, Probe::new("third", Flags::of(Flag::Move)));
        let mut log = Log::default();
        selector.tick(&mut log);
        assert_eq!(running(&selector), vec!["first"]);
    }

    #[test]
    fn non_interruptable_goal_keeps_its_flags() {
        let mut selector = GoalSelector::new();
        let mut panic = Probe::new("panic", Flags::of(Flag::Move));
        panic.interruptable = false;
        selector.add_goal(5, panic);
        selector.add_goal(1, Probe::new("urgent", Flags::of(Flag::Move)));
        let mut log = Log::default();
        selector.tick(&mut log);
        assert_eq!(running(&selector), vec!["panic"]);
    }

    #[test]
    fn disabled_flags_stop_and_block_goals() {
        let mut selector = GoalSelector::new();
        selector.add_goal(1, Probe::new("move", Flags::of(Flag::Move)));
        selector.add_goal(1, Probe::new("look", Flags::of(Flag::Look)));
        let mut log = Log::default();
        selector.tick(&mut log);
        assert_eq!(running(&selector), vec!["move", "look"]);
        selector.disable_control_flag(Flag::Move);
        log.0.clear();
        selector.tick(&mut log);
        assert_eq!(running(&selector), vec!["look"]);
        assert_eq!(log.0, vec!["stop move", "tick look"]);
        selector.set_control_flag(Flag::Move, true);
        selector.tick(&mut log);
        assert_eq!(running(&selector), vec!["move", "look"]);
    }

    #[test]
    fn goals_that_cannot_continue_release_their_lock() {
        let mut selector = GoalSelector::new();
        selector.add_goal(1, Probe::new("move", Flags::of(Flag::Move)));
        let mut log = Log::default();
        selector.tick(&mut log);
        selector.goals[0].goal.usable = false;
        selector.tick(&mut log);
        assert!(running(&selector).is_empty());
        assert!(selector.locked(Flag::Move).is_none());
    }

    #[test]
    fn partial_ticks_only_run_every_tick_goals() {
        let mut selector = GoalSelector::new();
        let mut float = Probe::new("float", Flags::of(Flag::Jump));
        float.every_tick = true;
        selector.add_goal(0, float);
        selector.add_goal(1, Probe::new("stroll", Flags::of(Flag::Move)));
        let mut log = Log::default();
        selector.tick(&mut log);
        log.0.clear();
        selector.tick_running_goals(&mut log, false);
        assert_eq!(log.0, vec!["tick float"]);
    }

    #[test]
    fn tick_delays_halve_unless_every_tick() {
        assert_eq!(reduced_tick_delay(120), 60);
        assert_eq!(adjusted_tick_delay(false, 41), 21);
        assert_eq!(adjusted_tick_delay(true, 41), 41);
    }
}
