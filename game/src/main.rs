#![no_std]
#![no_main]

use r_efi::efi;
use r_efi::efi::protocols::graphics_output as gop;
use utils::{locate_protocol, print, utf16_cstring};

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

enum GameState {
    Running,
    Won,
    Lost,
}

struct Game {
    game_state: GameState,
    ball: Ball,
    blocks: [Block; 2],
}

struct Screen {
    fb: *mut u32,
    width: isize,
    height: isize,
}

struct Block {
    x: isize,
    y: isize,
    prev_y: isize,
    width: isize,
    height: isize,
    color: u32,
}

struct Ball {
    x: isize,
    y: isize,
    vx: isize,
    vy: isize,
    radius: isize,
    color: u32,
}

struct AIState {
    cooldown: usize,
}

struct InputState {
    w: bool,
    s: bool,
}

unsafe fn fill_screen(screen: &Screen, color: u32) {
    unsafe {
        for y in 0..screen.height {
            for x in 0..screen.width {
                let idx = (y * screen.width + x) as usize;
                *screen.fb.add(idx) = color;
            }
        }
    }
}

unsafe fn draw_blocks(screen: &Screen, game: &Game) {
    unsafe {
        for block in &game.blocks {
            for y in block.y..block.y + block.height {
                for x in block.x..block.x + block.width {
                    let idx = (y * screen.width + x) as usize;
                    *screen.fb.add(idx) = block.color;
                }
            }
        }
    }
}

unsafe fn draw_ball(screen: &Screen, ball: &Ball) {
    unsafe {
        let r2 = ball.radius * ball.radius;

        for y in (ball.y - ball.radius)..=(ball.y + ball.radius) {
            for x in (ball.x - ball.radius)..=(ball.x + ball.radius) {
                let dx = x - ball.x;
                let dy = y - ball.y;

                if dx * dx + dy * dy <= r2 {
                    let idx = (y * screen.width + x) as usize;
                    *screen.fb.add(idx) = ball.color;
                }
            }
        }
    }
}

fn update_ball(screen: &Screen, game: &mut Game) {
    let ball = &mut game.ball;

    ball.x += ball.vx;
    ball.y += ball.vy;

    if ball.y - ball.radius <= 0 || ball.y + ball.radius >= screen.height {
        ball.vy = -ball.vy;
    }

    // paddle collision
    for block in &game.blocks {
        if ball.x + ball.radius >= block.x
            && ball.x - ball.radius <= block.x + block.width
            && ball.y + ball.radius >= block.y
            && ball.y - ball.radius <= block.y + block.height
        {
            ball.vx = -ball.vx;

            let paddle_vy = block.y - block.prev_y;

            ball.vy += paddle_vy / 5;
        }
    }
    ball.vy = ball.vy.clamp(-10, 10);

    if ball.x + ball.radius < 0 {
        game.game_state = GameState::Lost;
    } else if ball.x - ball.radius > screen.width {
        game.game_state = GameState::Won;
    }
}

fn update_paddle(game: &mut Game, screen: &Screen, input: &InputState) {
    let paddle = &mut game.blocks[0];

    paddle.prev_y = paddle.y;

    if input.w {
        paddle.y = paddle.y.saturating_sub(30);
    } else if input.s {
        paddle.y = (paddle.y + 30).min(screen.height - paddle.height);
    }
}

unsafe fn delay(bs: *mut efi::BootServices) {
    unsafe {
        ((*bs).stall)(16_666);
    }
}

unsafe fn poll_input(st: *mut efi::SystemTable, input: &mut InputState) {
    unsafe {
        input.w = false;
        input.s = false;
        let con_in = (*st).con_in;

        let mut key = core::mem::zeroed();

        loop {
            let status = ((*con_in).read_key_stroke)(con_in, &mut key);

            if status != efi::Status::SUCCESS {
                break;
            }

            match key.unicode_char as u8 as char {
                'w' => input.w = true,
                's' => input.s = true,
                _ => {}
            }
        }
    }
}

fn ai_update_paddle(game: &mut Game, screen: &Screen, ai: &mut AIState) {
    let player_center = game.blocks[0].y + (game.blocks[0].height / 2);
    let paddle = &mut game.blocks[1];
    paddle.prev_y = paddle.y;

    if ai.cooldown > 0 {
        ai.cooldown -= 1;
        return;
    }

    let ball_y = game.ball.y;
    let paddle_center = paddle.y + (paddle.height / 2);
    let tracking_ball = game.ball.vx > 0 && game.ball.x > screen.width / 2;

    let target_y = if tracking_ball {
        (ball_y + game.ball.vy * 5).clamp(paddle.height / 2, screen.height - paddle.height / 2)
    } else {
        player_center
    };

    let dead_zone = if tracking_ball { 16 } else { 24 };
    let step = if tracking_ball { 16 } else { 10 };

    if target_y < paddle_center - dead_zone {
        paddle.y = paddle.y.saturating_sub(step);
    } else if target_y > paddle_center + dead_zone {
        paddle.y = (paddle.y + step).min(screen.height - paddle.height);
    }

    ai.cooldown = if tracking_ball { 2 } else { 3 };
}

unsafe fn break_out_of_game(st: *mut efi::SystemTable, game: &GameState) -> Result<(), efi::Status> {
    unsafe {
        let rt = (*st).runtime_services;
        let name = utf16_cstring::<12>("PONG_RESULT");

        pub const PONG_GUID: efi::Guid = efi::Guid::from_fields(
            0x676E_6F6F,
            0x4F50,
            0x474E,
            0x00,
            0x00,
            &[0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
        );

        let result: u8 = match game {
            GameState::Won => 1,
            GameState::Lost => 0,
            GameState::Running => return Ok(()),
        };

        let attrs = efi::VARIABLE_BOOTSERVICE_ACCESS | efi::VARIABLE_RUNTIME_ACCESS;

        let status = ((*rt).set_variable)(
            name.as_ptr() as *mut u16,
            &PONG_GUID as *const _ as *mut _,
            attrs,
            core::mem::size_of_val(&result),
            &result as *const _ as *mut _,
        );

        if status.is_error() {
            return Err(status);
        }

        Ok(())
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn efi_main(_h: efi::Handle, st: *mut efi::SystemTable) -> efi::Status {
    unsafe {
        print(st, "Hello World!\r\n");

        let bs = (*st).boot_services;

        let gop = &*locate_protocol::<gop::Protocol>(bs, &gop::PROTOCOL_GUID as *const _ as *mut _)
            .unwrap();

        let mode = *gop.mode;
        let info = *(mode).info;

        let fb = gop.mode.as_ref().unwrap().frame_buffer_base as *mut u32;

        let screen = Screen {
            fb,
            width: info.horizontal_resolution as isize,
            height: info.vertical_resolution as isize,
        };

        // AARRGGBB
        let color = 0x000000ff;

        let block_a = Block {
            x: 0,
            y: screen.height / 2,
            prev_y: screen.height / 2,
            width: 15,
            height: 150,
            color: 0x00ffffff,
        };

        let block_b = Block {
            x: screen.width - 15,
            y: screen.height / 2,
            prev_y: screen.height / 2,
            width: 15,
            height: 150,
            color: 0x00ffffff,
        };

        let ball = Ball {
            x: (screen.width / 2) as isize,
            y: (screen.height / 2) as isize,
            vx: 10,
            vy: 4,
            radius: 10,
            color: 0x00ffffff,
        };

        let mut game = Game {
            game_state: GameState::Running,
            ball,
            blocks: [block_a, block_b],
        };

        let mut ai = AIState { cooldown: 2 };

        let mut input = InputState { w: false, s: false };

        loop {
            match game.game_state {
                GameState::Running => {}
                GameState::Lost => {
                    break;
                }
                GameState::Won => {
                    break;
                }
            }

            poll_input(st, &mut input);
            update_paddle(&mut game, &screen, &input);
            ai_update_paddle(&mut game, &screen, &mut ai);
            update_ball(&screen, &mut game);

            if !matches!(game.game_state, GameState::Running) {
                break;
            }

            fill_screen(&screen, color);
            draw_blocks(&screen, &game);
            draw_ball(&screen, &game.ball);

            delay(bs);
        }

        if break_out_of_game(st, &game.game_state).is_err() {
            print(st, "setvar fail\r\n");
        }
        efi::Status::SUCCESS
    }
}
