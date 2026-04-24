fn main() {
    const DOOM_DIR: &str = "doomgeneric/doomgeneric";
    const SOURCES: &[&str] = &[
        "dummy.c",
        "am_map.c",
        "doomdef.c",
        "doomstat.c",
        "dstrings.c",
        "d_event.c",
        "d_items.c",
        "d_iwad.c",
        "d_loop.c",
        "d_main.c",
        "d_mode.c",
        "d_net.c",
        "f_finale.c",
        "f_wipe.c",
        "g_game.c",
        "hu_lib.c",
        "hu_stuff.c",
        "info.c",
        "i_cdmus.c",
        "i_endoom.c",
        "i_joystick.c",
        "i_scale.c",
        "i_sound.c",
        "i_system.c",
        "i_timer.c",
        "memio.c",
        "m_argv.c",
        "m_bbox.c",
        "m_cheat.c",
        "m_config.c",
        "m_controls.c",
        "m_fixed.c",
        "m_menu.c",
        "m_misc.c",
        "m_random.c",
        "p_ceilng.c",
        "p_doors.c",
        "p_enemy.c",
        "p_floor.c",
        "p_inter.c",
        "p_lights.c",
        "p_map.c",
        "p_maputl.c",
        "p_mobj.c",
        "p_plats.c",
        "p_pspr.c",
        "p_saveg.c",
        "p_setup.c",
        "p_sight.c",
        "p_spec.c",
        "p_switch.c",
        "p_telept.c",
        "p_tick.c",
        "p_user.c",
        "r_bsp.c",
        "r_data.c",
        "r_draw.c",
        "r_main.c",
        "r_plane.c",
        "r_segs.c",
        "r_sky.c",
        "r_things.c",
        "sha1.c",
        "sounds.c",
        "statdump.c",
        "st_lib.c",
        "st_stuff.c",
        "s_sound.c",
        "tables.c",
        "v_video.c",
        "wi_stuff.c",
        "w_checksum.c",
        "w_file.c",
        "w_main.c",
        "w_wad.c",
        "z_zone.c",
        "w_file_stdc.c",
        "i_input.c",
        "i_video.c",
        "doomgeneric.c",
        "doomgeneric_uefi.c",
    ];

    let mut build = cc::Build::new();
    build.include(DOOM_DIR);
    build.warnings(false);

    let target = std::env::var("TARGET").expect("TARGET is set by cargo");

    if target == "x86_64-unknown-uefi" {
        build.compiler("clang");

        build.flag("--target=x86_64-unknown-windows-gnu");
        build.flag("-ffreestanding");
        build.flag("-fno-builtin");
        build.flag("-fno-stack-protector");
        build.flag("-nostdlibinc");
        build.flag("-nostdinc");
    }

    for source in SOURCES {
        let path = format!("{DOOM_DIR}/{source}");
        println!("cargo:rerun-if-changed={path}");
        build.file(path);
    }

    build.compile("doomgeneric");
}
