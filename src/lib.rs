#![feature(proc_macro_hygiene)]

use skyline::{hook, install_hook};
use skyline::nn::hid::NpadHandheldState;
use skyline::nn::ro::LookupSymbol;

use smash::app::{self, lua_bind::*};

pub static mut FIGHTER_MANAGER_ADDR: usize = 0;
pub static mut SHOULD_END_RESULT_SCREEN : bool = false;

pub fn handle_get_npad_state_start(
    state: *mut NpadHandheldState,
    _controller_id: *const u32,
) {
    unsafe {
        let mgr = *(FIGHTER_MANAGER_ADDR as *mut *mut app::FighterManager);
        if FighterManager::is_result_mode(mgr) && FighterManager::entry_count(mgr) > 0 {
            let actual_state = *state;
            let update_count = (*state).updateCount;
            // 0: A, 2: X (Right Joycon), 12: LeftDpad (Left Joycon)
            // TODO: Above^, should we support separate joycons? 
            let key_a: u64 = 0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001;
            let key_start: u64 = 0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0100_0000_0000;
            if (actual_state.Buttons & key_start) != 0 {
                SHOULD_END_RESULT_SCREEN = true;
            }

            if SHOULD_END_RESULT_SCREEN {
                use rand::{self, Rng};
                let mut rng = rand::thread_rng();
                // Need to space apart A-presses so it does not seem like we are holding the button.
                let n: u32 = rng.gen_range(0..3);
                if n == 1 {
                    (*state).Buttons |= key_a;
                }
            }
        } 
        
        if FighterManager::entry_count(mgr) == 0 {
            SHOULD_END_RESULT_SCREEN = false;
        }
    }
}

#[allow(improper_ctypes)]
extern "C" {
    fn add_nn_hid_hook(callback: fn(*mut NpadHandheldState,*const u32));
}

#[skyline::main(name = "results-screen")]
pub fn main() {
    std::thread::sleep(std::time::Duration::from_secs(20)); //makes it not crash on startup with arcrop bc ???
    println!("[results-screen] Installing hook...");
    unsafe {
        LookupSymbol(
            &mut FIGHTER_MANAGER_ADDR,
            "_ZN3lib9SingletonIN3app14FighterManagerEE9instance_E\u{0}"
                .as_bytes()
                .as_ptr(),
        );

        if (add_nn_hid_hook as *const ()).is_null() {
            panic!("The NN-HID hook plugin could not be found and is required to add NRO hooks. Make sure libnn_hid_hook.nro is installed.");
        }
        add_nn_hid_hook(handle_get_npad_state_start);
    }
}
