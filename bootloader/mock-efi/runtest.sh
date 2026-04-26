#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")"

# copy game.efi
cp ../../target/x86_64-unknown-uefi/debug/game.efi ./esp/game.efi

# copy linux kernel
cp /boot/vmlinuz-linux ./esp/vmlinuz
cp /boot/initramfs-linux.img ./esp/initrd.img

# copy bootloader
rm ./esp/EFI/BOOT/BOOTX64.EFI
cp ../../target/x86_64-unknown-uefi/debug/bootloader.efi ./esp/EFI/BOOT/BOOTX64.EFI

exec qemu-system-x86_64 -enable-kvm \
    -m 1G \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.4m.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.4m.fd \
    -drive format=raw,file=fat:rw:esp
