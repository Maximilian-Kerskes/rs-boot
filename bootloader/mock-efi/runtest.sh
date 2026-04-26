#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")"

cp /boot/vmlinuz-6.17.0-22-generic ./esp/vmlinuz
cp /boot/initrd.img ./esp/initrd.img

rm ./esp/EFI/BOOT/BOOTX64.EFI
cp ../target/x86_64-unknown-uefi/debug/rs-boot.efi ./esp/EFI/BOOT/BOOTX64.EFI

exec qemu-system-x86_64 -enable-kvm \
    -m 1G \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE_4M.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS_4M.fd \
    -drive format=raw,file=fat:rw:esp
