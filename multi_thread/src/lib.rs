#![no_std]
extern crate alloc;

use hashbrown::HashMap;
use io::*;
use gstd::{async_main, exec, msg, prelude::*, ActorId,};

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
    participants: HashMap<ActorId, u128>,
    thread_status: ThreadState,
    distributed_tokens: u128,
    graph_rep: HashMap<String, Vec<String>>
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
            participants: HashMap::new(),
            thread_status: ThreadState::Active,
            distributed_tokens: 0,
            graph_rep: HashMap::new(),
        }
    }

    async fn mint_thread_contract(&mut self, amount_tokens: u128) {
        let address_ft = addresft_state_mut();
        let payload = FTAction::Mint(amount_tokens);
        let _ = msg::send_delayed(address_ft.ft_program_id, payload, 0, 0);
    }

    async fn tokens_transfer_reward(&mut self, amount_tokens: u128, dest: ActorId) {
        let address_ft = addresft_state_mut();
        let payload = FTAction::Transfer{from: exec::program_id(), to: dest, amount: amount_tokens};
        let _ = msg::send_delayed(address_ft.ft_program_id, payload, 0, 0);
    }

    async fn tokens_transfer_pay(&mut self, amount_tokens: u128) {
        let address_ft = addresft_state_mut();
        if let Some(thread) = thread_state_mut().storage.get_mut(&self.id) {
            let payload = FTAction::Transfer{from: msg::source() , to: exec::program_id(), amount: amount_tokens};
            let _ = msg::send_delayed(address_ft.ft_program_id, payload, 0, 0);
            thread.participants.entry(msg::source()).or_insert(amount_tokens);
            thread.distributed_tokens += amount_tokens;
        };
    }

    fn find_winner_actor_id(&mut self) -> Option<&ActorId> {
        if let Some(thread) = thread_state_mut().storage.get_mut(&self.id) {
            let mut max_likes = 0;
            let mut actor_id_with_most_likes: Option<&ActorId> = None;

            for (_, reply) in &thread.replies {
                if reply.likes > max_likes {
                    max_likes = reply.likes;
                    actor_id_with_most_likes = Some(&reply.owner);
                }
            }
            actor_id_with_most_likes
        } else {
            None
        }
    }

    fn find_winner_post_id(&mut self) -> Option<&String> {
        if let Some(thread) = thread_state_mut().storage.get_mut(&self.id) {
            let mut max_likes = 0;
            let mut post_id_winner: Option<&String> = None;

            for (_, reply) in &thread.replies {
                if reply.likes > max_likes {
                    max_likes = reply.likes;
                    post_id_winner = Some(&reply.id);
                }
            }
            post_id_winner
        } else {
            None
        }
    }

    fn find_actor_id_by_post_id(&mut self, target_post_id: String) -> Option<&ActorId> {
        for (_, reply) in self.replies.iter() {
            if *reply.id == *target_post_id {
                return Some(&reply.owner)
            }
        }
        None
    }

    fn find_path_to_winner(&mut self, original_post_id: &String) -> Vec<ActorId> {
        let mut path_winners: Vec<ActorId> = Vec::new();

        // immediately add actor id of original post
        let original_post_actor_id = self.find_actor_id_by_post_id(original_post_id.clone())
            .expect("Actor ID not obtained correctly");
        path_winners.push(*original_post_actor_id);

        // find id of winner post
        let winner_id = self.find_winner_post_id()
            .expect("ActorID of winner not correctly found");

        // initially search for the winner
        let mut target_id = winner_id.clone();

        // Collect graph_rep values into a separate variable to end immutable borrow earlier
        let values: Vec<_> = self.graph_rep.values().cloned().collect();

        while target_id != *original_post_id {
            let mut found_reply = false;
            for adj_list in &values {
                for reply_id in adj_list {
                    // if reply id is the id of the target,
                    if *reply_id == target_id {
                        // find the actor id
                        let actor_id = self.find_actor_id_by_post_id(reply_id.clone())
                            .expect("ActorID not adequately found");
                        // append to the vector we will return.
                        path_winners.push(actor_id.clone());
                        target_id = reply_id.clone();
                        found_reply = true;
                        break;
                    }
                }
                if found_reply {
                    break;
                }
            }
        }
        path_winners
    }

    fn find_reply_by_id(&mut self, target_reply_id: String) -> Option<&mut ThreadReply> {
        let replies = &mut self.replies;
        for (_, reply) in replies {
            if reply.id == target_reply_id {
                return Some(reply);
            }
        }
        None
    }
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

            // Immediately push thread id to graph
            new_thread.graph_rep.insert(new_thread.id.clone(), Vec::new());

            // Mint one FT to the thread contract so it appears in the balance state of the FT contract
            new_thread.mint_thread_contract(1).await;

            // send delayed message to expire thread
            // let payload = ThreadAction::EndThread;
            // let delay = 1000;
            // let _end_thread_msg = msg::send_delayed(exec::program_id(), payload, 0, delay).expect("Delayed expiration msg was not successfully sent");

            // transfer a token
            new_thread.tokens_transfer_pay(1).await;

            // push new thread to vector of threads
            threads.insert(new_thread.id.clone(), new_thread);
        }

        ThreadAction::EndThread(end_thread_payload) => {
            if let Some(thread) = thread_state_mut().storage.get_mut(&end_thread_payload.thread_id) {
                thread.thread_status = ThreadState::Expired;
                let &winner = thread.find_winner_actor_id().expect("Winner not found");

                let distributed_tokens = thread.distributed_tokens;

                // calculate amount of tokens to distribute
                let tokens_for_abs_winner = distributed_tokens.clone() * 4 / 10;

                thread.tokens_transfer_reward(tokens_for_abs_winner, winner).await;
                let path_winners = thread.find_path_to_winner(&thread.id.clone());
                if !path_winners.is_empty() {
                    let tokens_for_each_path_winner = (distributed_tokens.clone() * 4 / 10) / path_winners.len() as u128;
                    for actor in path_winners {
                        thread.tokens_transfer_reward(tokens_for_each_path_winner, actor).await;
                    }
                }
            };
        }

        ThreadAction::AddReply(payload) => {
            if let Some(thread) = thread_state_mut().storage.get_mut(&payload.thread_id) {
                let _reply_user = thread.replies.entry(payload.reply_id.clone()).or_insert(ThreadReply {
                    id: payload.reply_id.clone(),
                    owner: msg::source(),
                    title: payload.title,
                    content: payload.content,
                    likes: 0,
                    reports: 0,
                });

                // create a new node
                thread.graph_rep.entry(payload.reply_id.clone()).or_insert_with(Vec::new);

                // push reply to the graph representation
                if let Some(adj_list) = thread.graph_rep.get_mut(&payload.referral_reply_id) {
                    adj_list.push(payload.reply_id.clone());
                }
            };
        }

        ThreadAction::LikeReply(payload) => {
            if let Some(thread) = thread_state_mut().storage.get_mut(&payload.thread_id) {
                let parsed_amount: u128 = payload.amount.parse().unwrap();
                thread.participants.entry(msg::source()).or_insert(parsed_amount);
                let reply = thread.find_reply_by_id(payload.reply_id).expect("Reply_id not get");
                reply.likes += 1;
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
                let io_thread = IoThread {
                    id: thread.id.clone(),
                    owner: thread.owner.clone(),
                    thread_type: thread.thread_type.clone(),
                    title: thread.title.clone(),
                    content: thread.content.clone(),
                    photo_url: thread.photo_url.clone(),
                    participants: thread.participants.iter().map(|(k, v)| (*k, v.clone())).collect(),
                    replies: thread.replies.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                    thread_status: thread.thread_status.clone(),
                    distributed_tokens: thread.distributed_tokens.clone(),
                    graph_rep: thread.graph_rep.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                };
                (key, io_thread)
            })
            .collect();

        Self { threads }
    }
}


