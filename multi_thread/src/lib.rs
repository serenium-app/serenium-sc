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
    thread_status: ThreadStatus,
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
            thread_status: ThreadStatus::Active,
            distributed_tokens: 0,
            graph_rep: HashMap::new(),
        }
    }

    async fn mint_thread_contract(&mut self, amount_tokens: u128) {
        let address_ft = addresft_state_mut();
        let payload = FTAction::Mint(amount_tokens);
        let _ = msg::send(address_ft.ft_program_id, payload, 0);
    }

    async fn transfer_tokens(&mut self, amount_tokens: u128) {
        let address_ft = addresft_state_mut();
        let transfer_payload = FTAction::Transfer {
            from: msg::source(),
            to: exec::program_id(),
            amount: amount_tokens
        };
        let _transfer = msg::send(address_ft.ft_program_id, transfer_payload, 0);
    }

    async fn burn_thread_contract(&mut self, amount_tokens: u128) {
        let address_ft = addresft_state_mut();
        let payload = FTAction::Burn(amount_tokens);
        let _ = msg::send(address_ft.ft_program_id, payload, 0);
    }

    async fn tokens_transfer_reward(&mut self, amount_tokens: u128, dest: ActorId) {
        let address_ft = addresft_state_mut();
        let payload = FTAction::Transfer{from: exec::program_id(), to: dest, amount: amount_tokens};
        let _ = msg::send(address_ft.ft_program_id, payload, 0);
    }

    async fn tokens_transfer_pay(&mut self, amount_tokens: u128) {
        self.mint_thread_contract(1).await;
        self.transfer_tokens(amount_tokens).await;
        self.burn_thread_contract(1).await;

        self.participants.entry(msg::source()).or_insert(amount_tokens);
        self.distributed_tokens += amount_tokens;
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

    fn find_path_to_winner(&mut self, original_post_id: &String) -> Vec<ActorId> {
        let mut path_winners: Vec<ActorId> = Vec::new();
        path_winners.push(self.owner);

        let winner_id = match self.find_winner_post_id() {
            Some(id) => id,
            None => return path_winners, // Or handle this scenario as needed
        };

        let mut target_id = winner_id.clone();
        let adjacency_lists: Vec<_> = self.graph_rep.values().cloned().collect();

        while target_id != *original_post_id {
            for adj_list in &adjacency_lists {
                if let Some(actor_id) = adj_list.iter()
                    .find(|&reply_id| reply_id == &target_id)
                    .and_then(|reply_id| self.replies.get(reply_id).map(|reply| reply.owner.clone()))
                {
                    path_winners.push(actor_id);
                    target_id = target_id.clone();
                    break;
                }
            }
        }

        path_winners
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

            // send delayed message to expire thread
            let payload = ThreadAction::EndThread(new_thread.id.clone());
            let delay = 1000;
            let _end_thread_msg = msg::send_delayed(exec::program_id(), payload, 0, delay).expect("Delayed expiration msg was not successfully sent");

            // transfer a token
            new_thread.tokens_transfer_pay(1).await;

            // push new thread to vector of threads
            threads.insert(new_thread.id.clone(), new_thread);

        }

        ThreadAction::EndThread(thread_id) => {
            if let Some(thread) = thread_state_mut().storage.get_mut(&thread_id) {
                thread.thread_status = ThreadStatus::Expired;
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

                thread.graph_rep.entry(payload.reply_id.clone()).or_insert_with(Vec::new);

                // push reply to the graph representation
                if let Some(adj_list) = thread.graph_rep.get_mut(&payload.referral_reply_id) {
                    adj_list.push(payload.reply_id.clone());
                }

                thread.tokens_transfer_pay(1).await;
            };
        }

        ThreadAction::LikeReply(payload) => {
            if let Some(thread) = thread_state_mut().storage.get_mut(&payload.thread_id) {
                let parsed_amount: u128 = payload.amount.parse().unwrap();
                thread.participants.entry(msg::source()).or_insert(parsed_amount);
                if let Some(reply) = thread.replies.get_mut(&payload.reply_id) {
                    reply.likes += parsed_amount;
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


