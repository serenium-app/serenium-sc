#![no_std]

use gmeta::{InOut, Metadata, Out};
use gstd::{collections::HashMap as GHashMap, msg, prelude::*, ActorId};
use io::{Post, PostId, Thread, ThreadReply};
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
    ) -> Option<Vec<(PostId, u128)>> {
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
                    return Some(all_replies_with_likes)
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
                StorageQueryReply::GraphRep(graph_rep) => return Some(graph_rep),
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
                StorageQueryReply::LikeHistoryOf(like_history) => return Some(like_history),
                _ => None,
            },
            Err(_) => None,
        }
    }

    pub fn trigger_reward_logic(thread: Thread) {
        let mut reward_logic_thread = RewardLogicThread::new(thread);
        reward_logic_thread.set_expired_thread_data();

        reward_logic_thread
            .expired_thread_data
            .as_mut()
            .expect("")
            .winner_reply = reward_logic_thread.find_winner_reply();

        reward_logic_thread
            .expired_thread_data
            .as_mut()
            .expect("")
            .top_liker_winner = reward_logic_thread.find_top_liker_winner();

        reward_logic_thread
            .expired_thread_data
            .as_mut()
            .expect("")
            .path_winners = reward_logic_thread.find_path_winners_actors();
    }
}

pub struct RewardLogicThread {
    pub thread_id: Option<PostId>,
    pub distributed_tokens: u128,
    pub graph_rep: Vec<(PostId, Vec<PostId>)>,
    pub replies: Vec<(PostId, ThreadReply)>,
    pub expired_thread_data: Option<ExpiredThread>,
}

impl RewardLogicThread {
    pub fn new(thread: Thread) -> Self {
        RewardLogicThread {
            thread_id: None,
            distributed_tokens: thread.distributed_tokens,
            graph_rep: thread.graph_rep,
            replies: thread.replies,
            expired_thread_data: None,
        }
    }

    pub fn set_expired_thread_data(&mut self) {
        let expired_thread_data = ExpiredThread::new();
        self.expired_thread_data = Some(expired_thread_data);
    }

    pub fn find_winner_reply(&self) -> Option<PostId> {
        // Iterate through the vector to find the ThreadReply with the most likes
        self.replies
            .iter()
            .max_by_key(|(_, reply)| reply.likes)
            .map(|(id, _)| *id) // Return the PostId of the winning reply
    }

    /// Finds the top liker winner based on the `like_history` of the winner's reply.
    ///
    /// This method retrieves the winner reply from `expired_thread_data`, then finds the entry in
    /// `self.replies` corresponding to the winner reply. It then iterates over the `like_history`
    /// associated with the winner reply to find the key (ActorId) corresponding to the entry with
    /// the highest value. If such an entry exists, it returns `Some(ActorId)` representing the key
    /// of the top liker winner. If `expired_thread_data` is not present, or if the winner reply is
    /// not available, or if the `like_history` is empty, it returns `None`.
    ///
    /// # Returns
    ///
    /// - `Some(ActorId)`: The key corresponding to the entry with the highest value in the
    ///   `like_history` of the winner's reply, if found.
    /// - `None`: If `expired_thread_data` is not present, or if the winner reply is not available,
    ///   or if the `like_history` is empty.
    pub fn find_top_liker_winner(&mut self) -> Option<ActorId> {
        // Retrieve the expired thread data
        self.expired_thread_data
            .as_ref()
            .and_then(|expired_thread_data| {
                // Retrieve the PostId of the winner reply
                expired_thread_data
                    .winner_reply
                    .and_then(|winner_reply_id| {
                        // Find the tuple in the vector of replies
                        self.replies
                            .iter()
                            .find(|(id, _)| *id == winner_reply_id)
                            .and_then(|(_, reply)| {
                                // Find the ActorId with the highest value in like history
                                reply
                                    .like_history
                                    .iter()
                                    .max_by_key(|&(_, likes)| *likes)
                                    .map(|(actor_id, _)| actor_id.clone())
                            })
                    })
            })
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
        let target = self
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

    /// Finds a path from the start node to the winner reply node in the graph and retrieves the owners of the posts in the path.
    ///
    /// Returns:
    /// - `Some(Vec<ActorId>)`: A vector representing the owners of the posts along the path from the start node to the winner reply node,
    ///                           where each element is an ActorId.
    /// - `None`: If no path is found from the start node to the winner reply node.
    pub fn find_path_winners_actors(&mut self) -> Option<Vec<ActorId>> {
        self.find_path_winners().map(|path_winners_post_id| {
            path_winners_post_id
                .iter()
                .filter_map(|post_id| {
                    self.replies
                        .iter()
                        .find(|(id, _)| id == post_id)
                        .map(|(_, reply)| reply.post_data.owner.clone())
                })
                .collect()
        })
    }
}

pub struct ExpiredThread {
    pub top_liker_winner: Option<ActorId>,
    pub path_winners: Option<Vec<ActorId>>,
    pub transaction_log: Option<Vec<(ActorId, u64)>>,
    pub winner_reply: Option<PostId>,
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
    TriggerRewardLogic(Thread),
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
