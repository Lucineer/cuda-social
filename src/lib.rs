/*!
# cuda-social

Social dynamics for agent fleets.

Intelligence isn't just individual — it's social. Ants build cities.
Wolves hunt in packs. Humans build civilizations. The group is
smarter than the individual.

- Social norms (implicit rules of behavior)
- Reputation systems (what others think of you)
- Group formation (clustering by affinity)
- Leadership emergence (natural authority from competence)
- Cooperation game theory (tit-for-tat, public goods)
- Conflict resolution (mediation, compromise)
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A social norm
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Norm {
    pub id: String,
    pub rule: String,
    pub compliance_rate: f64,    // how often agents follow this
    pub enforcement: f64,        // how strongly enforced [0, 1]
    pub context: String,
    pub created: u64,
    pub followers: Vec<String>,  // agent IDs that follow this norm
}

impl Norm {
    pub fn new(id: &str, rule: &str, context: &str) -> Self {
        Norm { id: id.to_string(), rule: rule.to_string(), compliance_rate: 0.5, enforcement: 0.5, context: context.to_string(), created: now(), followers: vec![] }
    }

    pub fn follow(&mut self, agent_id: &str) {
        if !self.followers.contains(&agent_id.to_string()) { self.followers.push(agent_id.to_string()); }
        self.compliance_rate = self.compliance_rate * 0.9 + 0.1;
    }

    pub fn violate(&mut self) {
        self.compliance_rate = (self.compliance_rate - 0.1).max(0.0);
    }
}

/// Reputation — what the fleet thinks about an agent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reputation {
    pub agent_id: String,
    pub competence: f64,        // how good at tasks
    pub reliability: f64,       // does what it says
    pub cooperativeness: f64,   // works well with others
    pub reputation_score: f64,  // composite
    pub endorsements: u32,
    pub complaints: u32,
}

impl Reputation {
    pub fn new(agent_id: &str) -> Self {
        Reputation { agent_id: agent_id.to_string(), competence: 0.5, reliability: 0.5, cooperativeness: 0.5, reputation_score: 0.5, endorsements: 0, complaints: 0 }
    }

    pub fn endorse(&mut self) {
        self.endorsements += 1;
        self.reputation_score = (self.reputation_score + 0.05).min(1.0);
    }

    pub fn complain(&mut self) {
        self.complaints += 1;
        self.reputation_score = (self.reputation_score - 0.1).max(0.0);
    }

    pub fn update_composite(&mut self) {
        self.reputation_score = self.competence * 0.3 + self.reliability * 0.3 + self.cooperativeness * 0.2 + (self.endorsements as f64 / (self.endorsements + self.complaints + 2) as f64) * 0.2;
    }

    pub fn is_reputable(&self, threshold: f64) -> bool { self.reputation_score >= threshold }
}

/// A social group
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SocialGroup {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
    pub leader: Option<String>,
    pub purpose: String,
    pub cohesion: f64,          // how unified the group is
    pub created: u64,
}

impl SocialGroup {
    pub fn new(id: &str, name: &str, purpose: &str) -> Self {
        SocialGroup { id: id.to_string(), name: name.to_string(), members: vec![], leader: None, purpose: purpose.to_string(), cohesion: 0.5, created: now() }
    }

    pub fn add_member(&mut self, agent_id: &str) {
        if !self.members.contains(&agent_id.to_string()) { self.members.push(agent_id.to_string()); }
    }

    pub fn remove_member(&mut self, agent_id: &str) {
        self.members.retain(|m| m != agent_id);
        if self.leader.as_deref() == Some(agent_id) { self.leader = None; }
    }

    pub fn size(&self) -> usize { self.members.len() }

    /// Elect leader by reputation (passed externally)
    pub fn elect_leader(&mut self, reputations: &HashMap<String, f64>) {
        self.leader = self.members.iter()
            .filter_map(|id| reputations.get(id).map(|r| (id.clone(), *r)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(id, _)| id);
    }
}

/// Cooperation strategy for game-theoretic interactions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CooperationStrategy {
    TitForTat,        // cooperate first, then mirror
    AlwaysCooperate,
    AlwaysDefect,
    Grudger,          // cooperate until first defection, then always defect
    GenerousTat,      // tit-for-tat but occasionally forgive
    Pavlov,           // win-stay, lose-shift
}

impl CooperationStrategy {
    /// Decide: cooperate (true) or defect (false)
    pub fn decide(&self, my_last_action: bool, their_last_action: bool, interaction_count: u32) -> bool {
        match self {
            CooperationStrategy::AlwaysCooperate => true,
            CooperationStrategy::AlwaysDefect => false,
            CooperationStrategy::TitForTat => if interaction_count == 0 { true } else { their_last_action },
            CooperationStrategy::Grudger => if their_last_action { true } else { false }, // once defected, always false
            CooperationStrategy::GenerousTat => {
                if interaction_count == 0 { return true; }
                if their_last_action { true } else { (interaction_count as f64).sqrt() * 0.2 > 0.5 } // occasionally forgive
            }
            CooperationStrategy::Pavlov => if interaction_count == 0 { true } else { my_last_action == their_last_action },
        }
    }
}

/// Conflict between agents
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Conflict {
    pub id: String,
    pub parties: Vec<String>,
    pub issue: String,
    pub severity: f64,
    pub timestamp: u64,
    pub resolved: bool,
    pub resolution: Option<String>,
}

/// The social system
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SocialSystem {
    pub norms: HashMap<String, Norm>,
    pub reputations: HashMap<String, Reputation>,
    pub groups: HashMap<String, SocialGroup>,
    pub strategies: HashMap<String, CooperationStrategy>,
    pub conflicts: Vec<Conflict>,
    pub default_strategy: CooperationStrategy,
}

impl SocialSystem {
    pub fn new() -> Self { SocialSystem { norms: HashMap::new(), reputations: HashMap::new(), groups: HashMap::new(), strategies: HashMap::new(), conflicts: vec![], default_strategy: CooperationStrategy::TitForTat } }

    /// Add norm
    pub fn add_norm(&mut self, norm: Norm) { self.norms.insert(norm.id.clone(), norm); }

    /// Record norm compliance
    pub fn comply(&mut self, norm_id: &str, agent_id: &str) {
        if let Some(norm) = self.norms.get_mut(norm_id) { norm.follow(agent_id); }
    }

    /// Record norm violation
    pub fn violate(&mut self, norm_id: &str, agent_id: &str) {
        if let Some(norm) = self.norms.get_mut(norm_id) { norm.violate(); }
    }

    /// Get or create reputation
    pub fn reputation(&mut self, agent_id: &str) -> &mut Reputation {
        self.reputations.entry(agent_id.to_string()).or_insert_with(|| Reputation::new(agent_id))
    }

    /// Endorse an agent
    pub fn endorse(&mut self, from: &str, to: &str) {
        self.reputation(to).endorse();
    }

    /// Complain about an agent
    pub fn complain(&mut self, from: &str, about: &str) {
        self.reputation(about).complain();
    }

    /// Create a group
    pub fn create_group(&mut self, id: &str, name: &str, purpose: &str) {
        self.groups.insert(id.to_string(), SocialGroup::new(id, name, purpose));
    }

    /// Join a group
    pub fn join_group(&mut self, group_id: &str, agent_id: &str) {
        if let Some(group) = self.groups.get_mut(group_id) { group.add_member(agent_id); }
    }

    /// Elect leaders for all groups
    pub fn elect_leaders(&mut self) {
        let rep_scores: HashMap<String, f64> = self.reputations.iter().map(|(id, r)| (id.clone(), r.reputation_score)).collect();
        for group in self.groups.values_mut() { group.elect_leader(&rep_scores); }
    }

    /// Set cooperation strategy
    pub fn set_strategy(&mut self, agent_id: &str, strategy: CooperationStrategy) {
        self.strategies.insert(agent_id.to_string(), strategy);
    }

    /// Decide cooperation
    pub fn cooperate_decision(&self, agent_id: &str, their_last: bool, my_last: bool, count: u32) -> bool {
        let strategy = self.strategies.get(agent_id).copied().unwrap_or(self.default_strategy);
        strategy.decide(my_last, their_last, count)
    }

    /// File a conflict
    pub fn file_conflict(&mut self, parties: Vec<String>, issue: &str, severity: f64) -> String {
        let id = format!("conflict_{}", self.conflicts.len());
        self.conflicts.push(Conflict { id: id.clone(), parties, issue: issue.to_string(), severity, timestamp: now(), resolved: false, resolution: None });
        id
    }

    /// Resolve a conflict
    pub fn resolve_conflict(&mut self, conflict_id: &str, resolution: &str) {
        if let Some(conflict) = self.conflicts.iter_mut().find(|c| c.id == conflict_id) {
            conflict.resolved = true;
            conflict.resolution = Some(resolution.to_string());
        }
    }

    /// Most reputable agent
    pub fn most_reputable(&self) -> Option<(String, f64)> {
        self.reputations.iter().max_by(|a, b| a.1.reputation_score.partial_cmp(&b.1.reputation_score).unwrap()).map(|(id, r)| (id.clone(), r.reputation_score))
    }

    /// Summary
    pub fn summary(&self) -> String {
        format!("SocialSystem: {} norms, {} reputations, {} groups, {} conflicts({} resolved), {} strategies",
            self.norms.len(), self.reputations.len(), self.groups.len(), self.conflicts.len(), self.conflicts.iter().filter(|c| c.resolved).count(), self.strategies.len())
    }
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_norm_compliance() {
        let mut norm = Norm::new("n1", "greet others", "social");
        norm.follow("alice");
        norm.follow("bob");
        assert_eq!(norm.followers.len(), 2);
        assert!(norm.compliance_rate > 0.5);
    }

    #[test]
    fn test_reputation_endorse() {
        let mut rep = Reputation::new("alice");
        rep.endorse();
        rep.endorse();
        assert_eq!(rep.endorsements, 2);
        assert!(rep.reputation_score > 0.5);
    }

    #[test]
    fn test_reputation_composite() {
        let mut rep = Reputation::new("alice");
        rep.competence = 0.9; rep.reliability = 0.8; rep.cooperativeness = 0.7;
        rep.endorse();
        rep.update_composite();
        assert!(rep.reputation_score > 0.7);
    }

    #[test]
    fn test_group_formation() {
        let mut ss = SocialSystem::new();
        ss.create_group("g1", "explorers", "explore maze");
        ss.join_group("g1", "alice");
        ss.join_group("g1", "bob");
        assert_eq!(ss.groups["g1"].size(), 2);
    }

    #[test]
    fn test_group_leader_election() {
        let mut ss = SocialSystem::new();
        ss.create_group("g1", "x", "y");
        ss.join_group("g1", "alice");
        ss.join_group("g1", "bob");
        ss.reputation("alice").reputation_score = 0.9;
        ss.reputation("bob").reputation_score = 0.3;
        ss.elect_leaders();
        assert_eq!(ss.groups["g1"].leader.as_deref(), Some("alice"));
    }

    #[test]
    fn test_tit_for_tat() {
        let strat = CooperationStrategy::TitForTat;
        assert!(strat.decide(true, true, 1)); // first interaction defaults to... actually count=1 means they cooperated
        assert!(!strat.decide(true, false, 1)); // mirror their defection
    }

    #[test]
    fn test_always_defect() {
        let strat = CooperationStrategy::AlwaysDefect;
        assert!(!strat.decide(true, true, 5));
    }

    #[test]
    fn test_pavlov() {
        let strat = CooperationStrategy::Pavlov;
        let result = strat.decide(true, true, 2); // both cooperated last time → cooperate
        assert!(result);
        let result2 = strat.decide(true, false, 2); // different → switch
        assert!(!result2);
    }

    #[test]
    fn test_conflict_resolution() {
        let mut ss = SocialSystem::new();
        let id = ss.file_conflict(vec!["a".into(), "b".into()], "resource dispute", 0.7);
        assert_eq!(ss.conflicts.len(), 1);
        ss.resolve_conflict(&id, "split evenly");
        assert!(ss.conflicts[0].resolved);
    }

    #[test]
    fn test_most_reputable() {
        let mut ss = SocialSystem::new();
        ss.reputation("alice").reputation_score = 0.9;
        ss.reputation("bob").reputation_score = 0.5;
        let best = ss.most_reputable();
        assert_eq!(best.unwrap().0, "alice");
    }

    #[test]
    fn test_norm_violation() {
        let mut norm = Norm::new("n1", "share resources", "social");
        norm.violate(); norm.violate();
        assert!(norm.compliance_rate < 0.5);
    }

    #[test]
    fn test_grudger() {
        let strat = CooperationStrategy::Grudger;
        assert!(strat.decide(true, true, 0)); // first = cooperate
        assert!(!strat.decide(true, false, 1)); // they defected once → forever defect
        assert!(!strat.decide(false, true, 2)); // even if they cooperate now
    }

    #[test]
    fn test_summary() {
        let ss = SocialSystem::new();
        let s = ss.summary();
        assert!(s.contains("0 norms"));
    }
}
