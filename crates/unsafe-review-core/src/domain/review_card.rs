use super::{
    CardId, Confidence, ContractEvidence, DischargeEvidence, HazardKind, MissingEvidence, Priority,
    ReachEvidence, RelatedTest, ReviewClass, SafetyObligation, UnsafeOperation, UnsafeSite,
    WitnessEvidence, WitnessRoute,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NextAction {
    pub summary: String,
    pub verify_commands: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewCard {
    pub id: CardId,
    pub class: ReviewClass,
    pub priority: Priority,
    pub confidence: Confidence,
    pub site: UnsafeSite,
    pub operation: UnsafeOperation,
    pub hazards: Vec<HazardKind>,
    pub obligations: Vec<SafetyObligation>,
    pub contract: ContractEvidence,
    pub discharge: DischargeEvidence,
    pub reach: ReachEvidence,
    pub witness: WitnessEvidence,
    pub missing: Vec<MissingEvidence>,
    pub routes: Vec<WitnessRoute>,
    pub next_action: NextAction,
    pub related_tests: Vec<RelatedTest>,
}
