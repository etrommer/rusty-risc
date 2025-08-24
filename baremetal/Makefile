SHELL = /bin/sh
ARCH = riscv32-unknown-elf
CC = $(ARCH)-gcc
LD = $(ARCH)-ld
OBJCOPY = $(ARCH)-objcopy
OBJDUMP = $(ARCH)-objdump

IDIR = inc
SDIR = src
BDIR = build
CFLAGS = -Wall -mcmodel=medany -march=rv32imafd -I $(IDIR) -Og -ffreestanding -fno-common -nostdlib -mno-relax
SFLAGS = -g -I $(IDIR)
LDFLAGS = -Wl,--gc-sections -Wl,-Map=kernel.map -nostdlib -nodefaultlibs -nostartfiles -mabi=ilp32
S_SRCS = $(wildcard $(SDIR)/*.s)
C_SRCS = $(wildcard $(SDIR)/*.c)
S_OBJS = $(S_SRCS:$(SDIR)/%.s=$(BDIR)/%_asm.o)
C_OBJS = $(C_SRCS:$(SDIR)/%.c=$(BDIR)/%.o)


all: clean disasm kernel.bin

kernel.bin: kernel.elf
	$(OBJCOPY) -S -I elf32-little -O binary kernel.elf kernel.bin

kernel.elf: $(S_OBJS) link.ld $(C_OBJS)
	$(CC) -T link.ld -o kernel.elf $(S_OBJS) $(C_OBJS) $(LDFLAGS)

$(BDIR)/%.o: $(SDIR)/%.c
	$(CC) $(CFLAGS) -c $< -o $@

$(BDIR)/%_asm.o: $(SDIR)/%.s
	$(CC) $(SFLAGS) -c $< -o $@

disasm: kernel.elf
	$(OBJDUMP) -d -S $< > disasm.s

clean:
	rm -f $(BDIR)/*_asm.o $(BDIR)/*.o kernel.elf kernel.bin

run: all
	qemu-system-riscv64 -M virt -kernel kernel.elf -bios none -serial stdio -display none -s -S

debug: all
	terminator -e "qemu-system-riscv64 -M virt -kernel kernel.elf -bios none -serial stdio -s -S" --new-tab
	terminator -e "riscv64-linux-gnu-gdb -x gdb.init" --new-tab

