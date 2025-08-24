#include <stdint.h>

static volatile uint8_t *uart = (void *)0x10000000;

static uint8_t putchar(uint8_t ch) {
    static uint8_t THR    = 0x00;
    static uint8_t LSR    = 0x05;
    static uint8_t LSR_RI = 0x40;

    while ((uart[LSR] & LSR_RI) == 0);
    return uart[THR] =  (uint8_t) ch;
}

void puts(const char *s) {
    while (*s) putchar(*s++);
    putchar('\n');
}

int strlen(const char *str) {
    int i;
    for (i = 0;str[i] != '\0';i++);
    return i;
}

void strrev(char *str) {
    int i;
    int sz = strlen(str);
    for (i = 0;i < sz / 2;i++) {
        char c = str[i];
        str[i] = str[sz - i - 1];
        str[sz - i - 1] = c;
    }
}

void main() {
    char * s = "Hello, RISC-V!";
    strrev(s);
    puts(s);
    strrev(s);
    puts(s);
    for(;;);
}
