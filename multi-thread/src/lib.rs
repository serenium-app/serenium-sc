#![no_std]
extern crate alloc;

use hashbrown::HashMap;
use io::*;
use gstd::{async_main, exec ,msg, prelude::*, ActorId, };

#[cfg(feature = "binary-vendor")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

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

    async fn mint_thread_contract(&mut self, amount_tokens: u128) {
        let address_ft = addresft_state_mut();
        let payload = FTAction::Mint(amount_tokens);
        let _ = msg::send(address_ft.ft_program_id, payload, 0);
    }

    async fn tokens_transfer_reward(&mut self, amount_tokens: u128, dest: ActorId) {
        let address_ft = addresft_state_mut();
        let payload = FTAction::Transfer{from: exec::program_id(), to: dest,amount: amount_tokens};
        let _ = msg::send(address_ft.ft_program_id, payload, 0);
    }

    async fn tokens_transfer_pay(&mut self, amount_tokens: u128) {
        let current_state = thread_state_mut();
        let address_ft = addresft_state_mut();
        let payload = FTAction::Transfer{from: msg::source() , to: exec::program_id(), amount: amount_tokens};
        let _ = msg::send(address_ft.ft_program_id, payload, 0);
        current_state.participants.entry(msg::source()).or_insert(amount_tokens);
        current_state.distributed_tokens += amount_tokens;
    }

    fn find_winner_actor_id(&mut self) -> Option<&ActorId> {
        let current_state = thread_state_mut();
        let mut max_likes = 0;
        let mut actor_id_with_most_likes: Option<&ActorId> = None;

        for (_, reply) in &current_state.replies {
            if reply.likes > max_likes {
                max_likes = reply.likes;
                actor_id_with_most_likes = Some(&reply.owner);
            }
        }
        actor_id_with_most_likes
    }

    fn find_winner_post_id(&mut self) -> Option<&String> {
        let current_state = thread_state_mut();
        let mut max_likes = 0;
        let mut post_id_winner: Option<&String> = None;

        for (_, reply) in &current_state.replies {
            if reply.likes > max_likes {
                max_likes = reply.likes;
                post_id_winner = Some(&reply.id);
            }
        }
        post_id_winner
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

static mut THREAD: Option<Thread> = None;

static mut ADDRESSFT:Option<InitFT> = None;

fn thread_state_mut() -> &'static mut Thread  {
    unsafe { THREAD.get_or_insert(Default::default()) }
}

fn addresft_state_mut() -> &'static mut InitFT {
    let addressft = unsafe { ADDRESSFT.as_mut()};
    unsafe { addressft.unwrap_unchecked() }
}

#[no_mangle]
extern "C" fn init () {
    let config: InitFT = msg::load().expect("Unable to decode InitFT");
    let thread = Thread {
        ..Default::default()
    };

    if config.ft_program_id.is_zero() {
        panic!("FT program address can't be 0");
    }

    let initft = InitFT {
        ft_program_id: config.ft_program_id
    };

    unsafe {
        ADDRESSFT = Some(initft);
    }

   unsafe { THREAD = Some(thread) };

}

#[async_main]
async fn main() {

    let action = msg::load().expect("Could not load Action");

    let _thread = unsafe { THREAD.get_or_insert(Thread::default()) };

    match action {
        ThreadAction::NewThread(init_thread) =>  {
            let new_thread = thread_state_mut();

            new_thread.id = init_thread.id;
            new_thread.owner = msg::source();
            new_thread.thread_type = init_thread.thread_type;
            new_thread.title = init_thread.title;
            new_thread.content = init_thread.content;
            new_thread.photo_url = init_thread.photo_url;
            new_thread.thread_status = ThreadState::Active;
            new_thread.graph_rep = HashMap::new();
            // Immediately push thread id to graph
            new_thread.graph_rep.insert(new_thread.id.clone(), Vec::new());

            // Mint one FT to the thread contract so it appears in the balance state of the FT contract
            new_thread.mint_thread_contract(1).await;

            // send delayed message to expire thread
            let payload = ThreadAction::EndThread;
            let delay = 60;
            let _end_thread_msg = msg::send_delayed(exec::program_id(), payload, 0, delay).expect("Delayed expiration msg was not successfully sent");

            // transfer a token
            new_thread.tokens_transfer_pay(1).await;
        }

        ThreadAction::EndThread => {
            let thread = thread_state_mut();
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
        }

        ThreadAction::AddReply(title, content, reply_id, referral_post_id) => {
            let thread = thread_state_mut();
            
            let _reply_user = thread.replies.entry(reply_id.clone()).or_insert(ThreadReply {
                id: reply_id.clone(),
                owner: msg::source(),
                title,
                content,
                likes: 0,
                reports: 0,
            });

            // create a new node
            thread.graph_rep.entry(reply_id.clone()).or_insert_with(Vec::new);

            // push reply to the graph representation
            if let Some(adj_list) = thread.graph_rep.get_mut(&referral_post_id) {
                adj_list.push(reply_id.clone());
            }
        }

        ThreadAction::LikeReply(amount, reply_id) => {
            let thread = thread_state_mut();
            thread.participants.entry(msg::source()).or_insert(amount);
            let reply = thread.find_reply_by_id(reply_id).expect("Reply_id not get");
            reply.likes += 1;
        }
    };
}

#[no_mangle]
extern fn state() {
        let thread = unsafe { THREAD.take().expect("Unexpected error in taking state") };
        msg::reply::<IoThread>(thread.into(), 0)
        .expect("Failed to encode or reply with `<ContractMetadata as Metadata>::State` from `state()`");
    }

#[derive(Decode, Encode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitFT {
    pub ft_program_id: ActorId,
}

impl From<Thread> for IoThread {
    fn from(value: Thread) -> Self {

    let Thread {
        id,
        owner,
        thread_type,
        title,
        content,
        photo_url,
        replies,
        participants,
        thread_status,
        distributed_tokens,
        graph_rep,
    } = value;

    let participants = participants.iter().map(|(k, v)| (*k, v.clone())).collect();
    let replies = replies.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let graph_rep = graph_rep.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

    Self {
        id,
        owner,
        title,
        thread_type,
        content,
        photo_url,
        replies,
        participants,
        thread_status,
        distributed_tokens,
        graph_rep
    }
}
}


