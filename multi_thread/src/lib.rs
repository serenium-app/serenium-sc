#![no_std]
extern crate alloc;

use hashbrown::HashMap;
use hashbrown::HashSet;
use io::*;
use gstd::{async_main, exec, msg, prelude::*, ActorId,};
use io::ThreadEvent::ThreadEnded;

#[cfg(feature = "binary-vendor")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[derive(Clone, Default)]
struct Threads {
    storage: HashMap<String, Thread>
}

impl Threads {
    fn new() -> Self {
        Threads {
            storage: HashMap::new(),
        }
    }
}

#[derive(Clone, Default)]
struct Thread {
    id: String,
    owner: ActorId,
    thread_type: ThreadType,
    title: String,
    content: String,
    photo_url: String,
    replies: HashMap<String, ThreadReply>,
    thread_status: ThreadStatus,
    distributed_tokens: u128,
    graph_rep: HashMap<String, Vec<String>>
}

#[derive(Clone, Default)]
pub struct ThreadReply {
    pub id: String,
    pub owner: ActorId,
    pub title: String,
    pub content: String,
    pub photo_url: String,
    pub likes: u128,
    pub reports: u128,
    pub like_history: HashMap<ActorId, u128>
}

#[derive(Clone, Default)]
pub struct ThreadExpired {
    winner_reply: ThreadReply,
    top_liker_winner: ActorId,
    path_winners: Vec<ActorId>,
    transaction_log: Vec<(ActorId, u128)>,
}

impl ThreadExpired {
    fn new(
        thread: &mut Thread
    ) -> Self {
        let mut instance = ThreadExpired {
            winner_reply: Default::default(),
            top_liker_winner: Default::default(),
            path_winners: Vec::new(),
            transaction_log: Vec::new(),
        };
        instance.find_winner_reply(thread);
        instance.find_likes_winner(thread);
        instance.find_path_to_winner(thread);
        instance
    }

    fn find_winner_reply(&mut self, thread: &Thread) {
        if let Some(thread) = thread_state_mut().storage.get(&thread.id) {
            let mut max_likes = 0;
            let mut winner_reply: Option<ThreadReply> = None;

            for (_, reply) in &thread.replies {
                if reply.likes > max_likes {
                    max_likes = reply.likes;
                    winner_reply = Some(reply.clone());
                }
            }
            self.winner_reply = winner_reply.unwrap();
        }
    }

    fn find_path_to_winner(&mut self, thread: &Thread) {
        self.path_winners.push(thread.owner.clone());

        let mut target_id = &self.winner_reply.id;

        // Convert adjacency lists into a HashMap for efficient lookups
        let adjacency_map: HashMap<&String, &Vec<String>> = thread.graph_rep.iter().collect();

        // Use a HashSet to keep track of visited nodes to avoid cycles
        let mut visited: HashSet<String> = HashSet::new();

        while *target_id != *thread.id {
            if let Some(adj_list) = adjacency_map.get(&target_id) {
                if let Some(reply_id) = adj_list.iter()
                    .find(|&reply_id| !visited.contains(reply_id))
                {
                    visited.insert(reply_id.clone());
                    if let Some(reply) = thread.replies.get(reply_id) {
                        let actor_id = reply.owner.clone();
                        self.path_winners.push(actor_id.clone());
                        target_id = &reply_id; // Set target_id to the reply_id for the next iteration
                    } else {
                        break; // Unable to retrieve reply or actor_id associated with it
                    }
                } else {
                    break; // No further nodes to explore
                }
            } else {
                break; // Invalid target_id or no adjacency list found
            }
        }
    }

    fn find_likes_winner(&mut self, thread: &Thread) {
        if let Some(winner_reply) = thread.replies.get(&self.winner_reply.id) {
            let mut max_actor: Option<ActorId> = None;
            let mut max_likes = 0;

            for (&actor, &num_likes) in &winner_reply.like_history {
                if num_likes > max_likes {
                    max_actor = Some(actor);
                    max_likes = num_likes;
                }
            }
            self.top_liker_winner = max_actor.unwrap();
        }
    }

    async fn process_path_winners(&mut self, thread: &Thread) {
        if !self.path_winners.is_empty() {
            let tokens_for_each_path_winner = thread.distributed_tokens.clone() * 3 / 10 / self.path_winners.len() as u128;
            for actor in &mut self.path_winners {
                self.transaction_log.push((*actor, tokens_for_each_path_winner))
            }
            for actor in &mut self.path_winners {
                transfer_tokens_reward(tokens_for_each_path_winner, *actor)
                    .await
                    .unwrap_or_else(|_| panic!("Token transfer failed!"));
            }
        }
    }
}

impl Thread {
    fn new(
        id: String,
        owner: ActorId,
        thread_type: ThreadType,
        title: String,
        content: String,
        photo_url: String,
    ) -> Self {
        Thread {
            id,
            owner,
            thread_type,
            title,
            content,
            photo_url,
            replies: HashMap::new(),
            thread_status: ThreadStatus::Active,
            distributed_tokens: 0,
            graph_rep: HashMap::new(),
        }
    }

    async fn mint_thread_contract(&mut self) {
        let address_ft = addresft_state_mut();
        let payload = FTAction::Mint(1);
        let _ = msg::send(address_ft.ft_program_id, payload, 0);
    }
}

async fn transfer_tokens_to_contract(amount_tokens: u128) -> Result<(), ()> {
    let address_ft = addresft_state_mut();
    let transfer_payload = FTAction::Transfer {
        from: msg::source(),
        to: exec::program_id(),
        amount: amount_tokens
    };
    let result =  msg::send_for_reply_as::<_, FTEvent>(address_ft.ft_program_id, transfer_payload,0,0).expect("Error in sending a message").await;
    let status = match result {
        Ok(event) => match event {
            FTEvent::Ok => Ok(()),
            _ => Err(()),
        },
        Err(_) => Err(())
    };
    status
}

async fn transfer_tokens_reward(amount_tokens: u128, dest: ActorId) -> Result<(), ()> {
    let address_ft = addresft_state_mut();
    let payload = FTAction::Transfer{from: exec::program_id(), to: dest, amount: amount_tokens};
    let result = msg::send_for_reply_as::<_, FTEvent>(address_ft.ft_program_id, payload, 0, 0)
        .expect("Error in sending a message")
        .await;
    let status = match result {
        Ok(event) => match event {
            FTEvent::Ok => Ok(()),
            _ => Err(()),
        },
        Err(_) => Err(())
    };
    status
}

static mut THREADS: Option<Threads> = None;

static mut ADDRESSFT:Option<InitFT> = None;

fn thread_state_mut() -> &'static mut Threads  {
    unsafe { THREADS.get_or_insert(Default::default()) }
}

fn addresft_state_mut() -> &'static mut InitFT {
    let addressft = unsafe { ADDRESSFT.as_mut()};
    unsafe { addressft.unwrap_unchecked() }
}

#[no_mangle]
extern "C" fn init () {
    let config: InitFT = msg::load().expect("Unable to decode InitFT");
    let threads: Threads = Threads::new();

    if config.ft_program_id.is_zero() {
        panic!("FT program address can't be 0");
    }

    let initft = InitFT {
        ft_program_id: config.ft_program_id
    };

    unsafe {
        ADDRESSFT = Some(initft);
    }

   unsafe { THREADS = Some(threads) };

}

#[async_main]
async fn main() {

    let action = msg::load().expect("Could not load Action");

    let _thread = unsafe { THREADS.get_or_insert(Threads::default()) };

    match action {
        ThreadAction::NewThread(init_thread) =>  {
            let threads = &mut thread_state_mut().storage;
            let mut new_thread: Thread = Thread::new(init_thread.id, msg::source(), init_thread.thread_type, init_thread.title, init_thread.content, init_thread.photo_url);

            new_thread.mint_thread_contract()
                .await;

            transfer_tokens_to_contract(1)
                .await
                .unwrap_or_else(|_| panic!("Token transfer failed!"));

            new_thread.distributed_tokens += 1;
            new_thread.graph_rep.insert(new_thread.id.clone(), Vec::new());

            // send delayed message to expire thread
            let payload = ThreadAction::EndThread(new_thread.id.clone());
            let delay = 1000;
            let _end_thread_msg = msg::send_delayed(exec::program_id(), payload, 0, delay).expect("Delayed expiration msg was not successfully sent");

            msg::reply(
                ThreadEvent::NewThreadCreated {
                    by: msg::source(),
                    id: new_thread.id.clone()
                },
                0)
                .unwrap();

            // push new thread to vector of threads
            threads.insert(new_thread.id.clone(), new_thread);
        }

        ThreadAction::EndThread(thread_id) => {
            if let Some(thread) = thread_state_mut().storage.get_mut(&thread_id) {
                thread.thread_status = ThreadStatus::Expired;
                let mut thread_expired = ThreadExpired::new(thread);

                transfer_tokens_reward(thread.distributed_tokens.clone() * 4 / 10, thread_expired.winner_reply.owner)
                    .await
                    .unwrap_or_else(|_| panic!("Token transfer failed!"));
                thread_expired.transaction_log.push((thread_expired.winner_reply.owner, thread.distributed_tokens.clone() * 4 / 10));

                transfer_tokens_reward(thread.distributed_tokens.clone() * 4 / 10, thread_expired.top_liker_winner.clone())
                    .await
                    .unwrap_or_else(|_| panic!("Token transfer failed!"));
                thread_expired.transaction_log.push((thread_expired.top_liker_winner, thread.distributed_tokens.clone() * 4 / 10));

                thread_expired.process_path_winners(thread).await;

                msg::reply(
                    ThreadEnded {
                        thread_id: thread.id.clone(),
                        transfers: thread_expired.transaction_log
                    }
                    , 0
                    )
                    .unwrap();
            }
        }

        ThreadAction::AddReply(payload) => {
            if let Some(thread) = thread_state_mut().storage.get_mut(&payload.thread_id) {
                transfer_tokens_to_contract(1)
                    .await
                    .unwrap_or_else(|_| panic!("Token transfer failed!"));

                let _reply_user = thread.replies.entry(payload.reply_id.clone()).or_insert(ThreadReply {
                    id: payload.reply_id.clone(),
                    owner: msg::source(),
                    title: payload.title,
                    content: payload.content,
                    photo_url: payload.photo_url,
                    likes: 0,
                    reports: 0,
                    like_history: HashMap::new()
                });

                thread.graph_rep.entry(payload.reply_id.clone()).or_insert_with(Vec::new);

                // push reply to the graph representation
                if let Some(adj_list) = thread.graph_rep.get_mut(&payload.referral_reply_id) {
                    adj_list.push(payload.reply_id.clone());
                }

                thread.distributed_tokens += 1;

                msg::reply(
                    ThreadEvent::ReplyAdded {
                        by: msg::source(),
                        id: _reply_user.id.clone(),
                        on_thread: thread.id.clone()
                    },
                    0)
                    .unwrap();
            };
        }

        ThreadAction::LikeReply(payload) => {
            if let Some(thread) = thread_state_mut().storage.get_mut(&payload.thread_id) {
                if let Some(reply) = thread.replies.get_mut(&payload.reply_id) {
                    if msg::source() == reply.owner {
                        panic!("User cannot like its own reply");
                    }

                    transfer_tokens_to_contract(payload.amount)
                        .await
                        .unwrap_or_else(|_| panic!("Token transfer failed!"));

                    reply.likes += payload.amount;
                    reply.like_history
                        .entry(msg::source())
                        .and_modify(|likes| *likes += payload.amount)
                        .or_insert(payload.amount);

                    thread.distributed_tokens += 1;

                    msg::reply(
                        ThreadEvent::ReplyLiked {
                            by: msg::source(),
                            like_count: payload.amount,
                            on_reply: reply.id.clone(),
                            on_thread: thread.id.clone()
                        },
                        0)
                        .unwrap();
                };
            };
        }
    };
}

#[no_mangle]
extern fn state() {
        let thread = unsafe { THREADS.take().expect("Unexpected error in taking state") };
        msg::reply::<IoThreads>(thread.into(), 0)
        .expect("Failed to encode or reply with `<ContractMetadata as Metadata>::State` from `state()`");
    }

#[derive(Decode, Encode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitFT {
    pub ft_program_id: ActorId,
}

impl From<Threads> for IoThreads {
    fn from(value: Threads) -> Self {
        let Threads { storage } = value;

        // Convert each Thread to IoThread and collect into a Vec<(String, IoThread)>
        let threads: Vec<(String, IoThread)> = storage
            .into_iter()
            .map(|(key, thread)| {
                let io_replies: Vec<(String, IoThreadReply)> = thread.replies.iter()
                    .map(|(k, v)| {
                        // Conversion for IoThreadReply, adjust as needed
                        let io_reply = IoThreadReply {
                            id: v.id.clone(),
                            owner: v.owner.clone(),
                            title: v.title.clone(),
                            content: v.content.clone(),
                            photo_url: v.photo_url.clone(),
                            likes: v.likes,
                            reports: v.reports,
                            // Convert like_history HashMap<ActorId, u128> to Vec<(ActorId, u128)>
                            like_history: v.like_history.iter().map(|(k, v)| (*k, *v)).collect(),
                        };
                        (k.clone(), io_reply)
                    })
                    .collect();

                let io_thread = IoThread {
                    id: thread.id.clone(),
                    owner: thread.owner.clone(),
                    thread_type: thread.thread_type.clone(),
                    title: thread.title.clone(),
                    content: thread.content.clone(),
                    photo_url: thread.photo_url.clone(),
                    replies: io_replies,
                    thread_status: thread.thread_status.clone(),
                    distributed_tokens: thread.distributed_tokens.clone(),
                    graph_rep: thread.graph_rep.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                };
                (key, io_thread)
            })
            .collect();
        Self { threads }
    }
}


