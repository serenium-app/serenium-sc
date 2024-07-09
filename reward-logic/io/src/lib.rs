#![no_std]

use gmeta::{InOut, Metadata, Out};
use gstd::{collections::HashMap as GHashMap, msg, prelude::*, ActorId};
use io::PostId;
use sharded_fungible_token_io::{FTokenEvent, LogicAction};
use storage_io::{StorageQuery, StorageQueryReply};

#[derive(Default, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct RewardLogic {
    pub admin: Option<ActorId>,
    pub address_ft: Option<ActorId>,
    pub address_logic: Option<ActorId>,
    pub address_storage: Option<ActorId>,
}

impl RewardLogic {
    pub fn new() -> Self {
        RewardLogic {
            admin: None,
            address_ft: None,
            address_logic: None,
            address_storage: None,
        }
    }

    pub async fn fetch_all_replies_with_likes(
        &mut self,
        thread_id: PostId,
    ) -> Option<Vec<(PostId, ActorId, u128)>> {
        let res = msg::send_for_reply_as::<_, StorageQueryReply>(
            self.address_storage.expect(""),
            StorageQuery::AllRepliesWithLikes(thread_id),
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(event) => match event {
                StorageQueryReply::AllRepliesWithLikes(all_replies_with_likes) => {
                    Some(all_replies_with_likes)
                }
                _ => None,
            },
            Err(_) => None,
        }
    }

    pub async fn fetch_graph_rep(
        &mut self,
        thread_id: PostId,
    ) -> Option<Vec<(PostId, Vec<PostId>)>> {
        let res = msg::send_for_reply_as::<_, StorageQueryReply>(
            self.address_storage.expect(""),
            StorageQuery::GraphRep(thread_id),
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(event) => match event {
                StorageQueryReply::GraphRep(graph_rep) => Some(graph_rep),
                _ => None,
            },
            Err(_) => None,
        }
    }

    pub async fn fetch_like_history(
        &mut self,
        thread_id: PostId,
        reply_id: PostId,
    ) -> Option<Vec<(ActorId, u128)>> {
        let res = msg::send_for_reply_as::<_, StorageQueryReply>(
            self.address_storage.expect(""),
            StorageQuery::LikeHistoryOf(thread_id, reply_id),
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(event) => match event {
                StorageQueryReply::LikeHistoryOf(like_history) => Some(like_history),
                _ => None,
            },
            Err(_) => None,
        }
    }

    pub async fn fetch_distributed_tokens(&mut self, thread_id: PostId) -> Option<u128> {
        let res = msg::send_for_reply_as::<_, StorageQueryReply>(
            self.address_storage.expect(""),
            StorageQuery::DistributedTokens(thread_id),
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(event) => match event {
                StorageQueryReply::DistributedTokens(distributed_tokens) => {
                    Some(distributed_tokens)
                }
                _ => None,
            },
            Err(_) => None,
        }
    }

    pub async fn trigger_reward_logic(&mut self, thread_id: PostId) {
        let _reward_logic_thread = RewardLogicThread::new(self, thread_id).await;
    }
}

pub struct RewardLogicThread {
    pub thread_id: Option<PostId>,
    pub distributed_tokens: u128,
    pub graph_rep: Vec<(PostId, Vec<PostId>)>,
    pub all_replies_with_likes: Vec<(PostId, ActorId, u128)>,
    pub winner_reply_like_history: Vec<(ActorId, u128)>,
    pub expired_thread_data: Option<ExpiredThread>,
}

impl RewardLogicThread {
    /// Constructs a new `RewardLogicThread` and initializes it with fetched data.
    ///
    /// # Parameters
    ///
    /// - `self_ref`: A reference to the calling object, providing methods for fetching data.
    /// - `thread_id`: The ID of the thread to fetch data for.
    ///
    /// # Returns
    ///
    /// A new instance of `RewardLogicThread` initialized with fetched data.
    pub async fn new(self_ref: &mut RewardLogic, thread_id: PostId) -> Self {
        let mut reward_logic_thread = RewardLogicThread {
            thread_id: Some(thread_id),
            distributed_tokens: 0,
            graph_rep: Vec::new(),
            all_replies_with_likes: Vec::new(),
            winner_reply_like_history: Vec::new(),
            expired_thread_data: None,
        };

        reward_logic_thread.set_expired_thread_data();

        // Fetch distributed tokens
        reward_logic_thread.distributed_tokens = self_ref
            .fetch_distributed_tokens(thread_id)
            .await
            .expect("Error in fetching the distributed tokens");

        // Fetch reward logic thread data here
        reward_logic_thread.all_replies_with_likes = self_ref
            .fetch_all_replies_with_likes(thread_id)
            .await
            .expect("Error in fetching all replies with likes");

        reward_logic_thread
            .expired_thread_data
            .as_mut()
            .expect("")
            .winner_reply = reward_logic_thread.find_winner_reply();

        // Fetch like history of winner reply
        let (reply_id, _, _) = reward_logic_thread
            .expired_thread_data
            .as_mut()
            .expect("")
            .winner_reply
            .expect("Winner reply not found");
        reward_logic_thread.winner_reply_like_history = self_ref
            .fetch_like_history(thread_id, reply_id)
            .await
            .expect("");

        reward_logic_thread
            .expired_thread_data
            .as_mut()
            .expect("")
            .top_liker_winner = reward_logic_thread.find_top_liker_winner();

        // Fetch graph rep
        reward_logic_thread.graph_rep = self_ref
            .fetch_graph_rep(thread_id)
            .await
            .expect("Error in fetching thread's graph rep");

        // Find path winners
        // reward_logic_thread
        //     .expired_thread_data
        //     .as_mut()
        //     .expect("")
        //     .path_winners = reward_logic_thread.find_path_winners_tokens();

        // Distribute rewards

        reward_logic_thread
    }

    pub fn set_expired_thread_data(&mut self) {
        let expired_thread_data = ExpiredThread::new();
        self.expired_thread_data = Some(expired_thread_data);
    }

    pub fn find_winner_reply(&self) -> Option<(PostId, ActorId, u128)> {
        let tokens = (self.distributed_tokens * 4) / 10;

        self.all_replies_with_likes
            .iter()
            .max_by_key(|(_, _, likes)| likes)
            .map(|(reply_id, actor_id, _)| (*reply_id, *actor_id, tokens)) // Return the PostId and ActorId of the winning reply
    }

    /// Finds the `ActorId` of the actor who has given the most likes to the winner has given the most likes.
    ///
    /// This function iterates through the `winner_reply_like_history` collection,
    /// which stores pairs of `ActorId` and the number of likes given by that actor.
    /// It returns the `ActorId` of the actor with the highest number of likes given.
    ///
    /// # Returns
    ///
    /// - `Some(ActorId)`: The `ActorId` of the actor who has given the most likes, if the collection is not empty.
    /// - `None`: If the `winner_reply_like_history` collection is empty.
    ///
    /// ```
    pub fn find_top_liker_winner(&mut self) -> Option<(ActorId, u128)> {
        let tokens = (self.distributed_tokens * 3) / 10;

        self.winner_reply_like_history
            .iter()
            .max_by_key(|&(_actor_id, likes_given)| *likes_given)
            .map(|(actor_id, _likes_given)| (*actor_id, tokens))
    }

    /// Finds a path from the start node to the winner reply node in the graph.
    ///
    /// Returns:
    /// - `Some(Vec<PostId>)`: A vector representing the path from the start node to the winner reply node,
    ///                          where each element is a PostId.
    /// - `None`: If no path is found from the start node to the winner reply node.
    ///
    /// # Panics
    ///
    /// This method will panic if:
    /// - `thread_id` is None, indicating that the start node is not set.
    /// - `expired_thread_data` is None, indicating that the winner reply node is not set.
    /// - `winner_reply` within `expired_thread_data` is None, indicating that the winner reply node is not set.
    ///
    /// ```
    pub fn find_path_winners(&self) -> Option<Vec<PostId>> {
        let start = self.thread_id.expect("Thread ID is not set.");
        let (target, _target_actor, _) = self
            .expired_thread_data
            .as_ref()
            .expect("Expired thread data is not set.")
            .winner_reply
            .expect("Winner reply is not set.");

        let mut visited = collections::HashSet::new();
        let mut queue = collections::VecDeque::new();
        let mut path = GHashMap::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(node) = queue.pop_front() {
            if node == target {
                // Reconstruct path
                let mut current = node;
                let mut result = Vec::new();
                while let Some(&prev) = path.get(&current) {
                    result.push(current);
                    current = prev;
                }
                result.push(start);
                result.reverse();
                return Some(result);
            }

            // Retrieve neighbors from the adjacency list
            if let Some((_, neighbors)) = self.graph_rep.iter().find(|(id, _)| *id == node) {
                for &neighbor in neighbors {
                    if visited.insert(neighbor) {
                        queue.push_back(neighbor);
                        path.insert(neighbor, node);
                    }
                }
            }
        }

        None
    }

    pub fn find_path_winners_tokens(&self) -> Option<(Vec<PostId>, u128)> {
        let path_winners: Vec<PostId> = self.find_path_winners().expect("");
        let tokens: u128 = ((self.distributed_tokens * 3) / 10) / path_winners.len() as u128;
        Some((path_winners, tokens))
    }

    pub async fn transfer_tokens(
        &mut self,
        ft_address_id: ActorId,
        amount: u128,
        sender: ActorId,
        recipient: ActorId,
    ) -> Result<(), ()> {
        let res = msg::send_for_reply_as::<_, FTokenEvent>(
            ft_address_id,
            LogicAction::Transfer {
                sender,
                recipient,
                amount,
            },
            0,
            0,
        )
        .expect("")
        .await;

        match res {
            Ok(event) => match event {
                FTokenEvent::Ok => Ok(()),
                _ => Err(()),
            },
            Err(_) => Err(()),
        }
    }

    // pub async fn distribute_rewards(&mut self, address_ft: ActorId) {
    //     let (_reply_id, actor_id) = self
    //         .expired_thread_data
    //         .as_ref()
    //         .expect("")
    //         .winner_reply
    //         .expect("");
    //     // Distribute reward to winner reply
    //     self.transfer_tokens(address_ft).unwrap()
    // }
}

impl Default for RewardLogicThread {
    /// Provides a default instance of `RewardLogicThread`.
    ///
    /// # Returns
    ///
    /// A new instance of `RewardLogicThread` with all fields initialized to `None` or default values.
    fn default() -> Self {
        RewardLogicThread {
            thread_id: None,
            distributed_tokens: 0,
            graph_rep: Vec::new(),
            all_replies_with_likes: Vec::new(),
            winner_reply_like_history: Vec::new(),
            expired_thread_data: None,
        }
    }
}

pub struct ExpiredThread {
    pub top_liker_winner: Option<(ActorId, u128)>,
    pub path_winners: Option<(Vec<ActorId>, u128)>,
    pub transaction_log: Option<Vec<(ActorId, u128)>>,
    pub winner_reply: Option<(PostId, ActorId, u128)>,
}

impl ExpiredThread {
    pub fn new() -> Self {
        ExpiredThread {
            top_liker_winner: None,
            path_winners: None,
            transaction_log: None,
            winner_reply: None,
        }
    }
}

impl Default for ExpiredThread {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum RewardLogicAction {
    AddAddressFT(ActorId),
    AddAddressLogic(ActorId),
    AddAddressStorage(ActorId),
    TriggerRewardLogic(PostId),
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum RewardLogicEvent {
    FTAddressAdded,
    LogicAddressAdded,
    StorageAddressAdded,
    RewardLogicTriggered,
}

pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init = ();
    type Handle = InOut<RewardLogicAction, RewardLogicEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = Out<RewardLogic>;
}
