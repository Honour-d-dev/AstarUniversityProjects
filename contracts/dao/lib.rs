#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod dao {
    use super::ensure;
    use ink::storage::Mapping;
    // use openbrush::contracts::traits::psp22::*;
    use scale::{
        Decode,
        Encode,
    };

    #[derive(Encode, Decode)]
    #[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq, scale_info::TypeInfo))]
    pub enum VoteType {
        // to implement
        For,
        Against,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum GovernorError {
        // to implement
        AmountShouldNotBeZero,
        DurationError,
        ProposalNotFound,
        ProposalAlreadyExecuted,
        ProposalNotAccepted,
        VotePeriodEnded,
        AlreadyVoted,
        QuorumNotReached,
    }

    #[derive(Encode, Decode)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink::storage::traits::StorageLayout
        )
    )]
    pub struct Proposal {
        // to implement
        to: AccountId,
        vote_start: u64,
        vote_end: u64,
        executed: bool,
        amount: Balance,
    }

    #[derive(Encode, Decode, Default)]
    #[cfg_attr(
        feature = "std",
        derive(
            Debug,
            PartialEq,
            Eq,
            scale_info::TypeInfo,
            ink::storage::traits::StorageLayout
        )
    )]
    pub struct ProposalVote {
        // to implement
        for_votes: u8,
        against_votes: u8,
    }

    pub type ProposalId = u64;

    #[ink(storage)]
    pub struct Governor {
        // to implement
        total_proposals: u64,
        proposals: Mapping<ProposalId, Proposal>,
        proposals_vote: Mapping<Proposal, ProposalVote>,
        votes: Mapping<(ProposalId, AccountId), ()>,
        governance_token: AccountId,
        quorum: u8,
    }

    impl Governor {
        #[ink(constructor, payable)]
        pub fn new(governance_token: AccountId, quorum: u8) -> Self {
            Self {
                total_proposals: 0,
                proposals: Mapping::default(),
                proposals_vote: Mapping::default(),
                votes: Mapping::default(),
                governance_token,
                quorum,
            }
        }

        #[ink(message)]
        pub fn propose(
            &mut self,
            to: AccountId,
            amount: Balance,
            duration: u64,
        ) -> Result<(), GovernorError> {
            ensure!(amount != 0, GovernorError::AmountShouldNotBeZero);
            ensure!(duration != 0, GovernorError::DurationError);

            let proposal = Proposal {
                to,
                vote_start: self.env().block_timestamp(),
                vote_end: (self.env().block_timestamp() + (duration * 60)),
                executed: false,
                amount,
            };

            self.proposals.insert(self.total_proposals, &proposal);
            self.proposals_vote
                .insert(proposal, &ProposalVote::default());
            self.total_proposals += 1;

            Ok(())
        }

        #[ink(message)]
        pub fn vote(
            &mut self,
            proposal_id: ProposalId,
            vote: VoteType,
        ) -> Result<(), GovernorError> {
            let voter = self.env().caller();

            ensure!(
                !self.votes.contains((&proposal_id, voter)),
                GovernorError::AlreadyVoted
            );

            ensure!(
                self.proposals.contains(&proposal_id),
                GovernorError::ProposalNotFound
            );

            let proposal = self.proposals.get(&proposal_id).unwrap();
            self.votes.insert((&proposal_id, voter), &());
            let mut proposal_vote = self.proposals_vote.get(&proposal).unwrap();
            match vote {
                VoteType::For => {
                    proposal_vote.for_votes += 1;
                    Ok(())
                }
                VoteType::Against => {
                    proposal_vote.against_votes += 1;
                    Ok(())
                }
            }
        }

        #[ink(message)]
        pub fn execute(&mut self, proposal_id: ProposalId) -> Result<(), GovernorError> {
            ensure!(
                self.proposals.contains(&proposal_id),
                GovernorError::ProposalNotFound
            );
            let mut proposal = self.proposals.get(&proposal_id).unwrap();
            ensure!(!proposal.executed, GovernorError::ProposalAlreadyExecuted);

            let proposal_vote = self.proposals_vote.get(&proposal).unwrap();
            ensure!(
                proposal_vote.for_votes >= proposal_vote.against_votes,
                GovernorError::ProposalNotAccepted
            );

            ensure!(
                proposal_vote.for_votes >= self.quorum,
                GovernorError::QuorumNotReached
            );

            self.env()
                .transfer(proposal.to, proposal.amount)
                .unwrap_or_else(|err| panic!("transfer failed: {:?}", err));
            proposal.executed = true;
            Ok(())
        }

        #[ink(message)]
        pub fn next_proposal_id(&self) -> u64 {
            self.total_proposals
        }

        #[ink(message)]
        pub fn get_proposal(&self, id: ProposalId) -> Option<Proposal> {
            self.proposals.get(id)
        }

        // used for test
        #[ink(message)]
        pub fn now(&self) -> u64 {
            self.env().block_timestamp()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn create_contract(initial_balance: Balance) -> Governor {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), initial_balance);
            Governor::new(AccountId::from([0x01; 32]), 50)
        }

        fn contract_id() -> AccountId {
            ink::env::test::callee::<ink::env::DefaultEnvironment>()
        }

        fn default_accounts(
        ) -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts::<ink::env::DefaultEnvironment>()
        }

        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(sender);
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
                account_id, balance,
            )
        }

        #[ink::test]
        fn propose_works() {
            let accounts = default_accounts();
            let mut governor = create_contract(1000);
            assert_eq!(
                governor.propose(accounts.django, 0, 1),
                Err(GovernorError::AmountShouldNotBeZero)
            );
            assert_eq!(
                governor.propose(accounts.django, 100, 0),
                Err(GovernorError::DurationError)
            );
            let result = governor.propose(accounts.django, 100, 1);
            assert_eq!(result, Ok(()));
            let proposal = governor.get_proposal(0).unwrap();
            let now = governor.now();
            assert_eq!(
                proposal,
                Proposal {
                    to: accounts.django,
                    amount: 100,
                    vote_start: 0,
                    vote_end: now + 1 * 60,
                    executed: false,
                }
            );
            assert_eq!(governor.next_proposal_id(), 1);
        }

        #[ink::test]
        fn quorum_not_reached() {
            let mut governor = create_contract(1000);
            let result = governor.propose(AccountId::from([0x02; 32]), 100, 1);
            assert_eq!(result, Ok(()));
            let execute = governor.execute(0);
            assert_eq!(execute, Err(GovernorError::QuorumNotReached));
        }
    }
}

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
#[macro_export]
macro_rules! ensure {
    ( $x:expr, $y:expr $(,)? ) => {{
        if !$x {
            return Err($y.into())
        }
    }};
}
