#ifndef LRE_EXEC_BACKTRRACK_H
#define LRE_EXEC_BACKTRRACK_H

#ifndef FALSE
enum {
    FALSE = 0,
    TRUE = 1,
};
#endif

typedef enum {
#define DEF(id, size) REOP_ ## id,
#include "libregexp-opcode.h"
#undef DEF
    REOP_COUNT,
} REOPCodeEnum;

typedef enum {
    RE_EXEC_STATE_SPLIT,
    RE_EXEC_STATE_LOOKAHEAD,
    RE_EXEC_STATE_NEGATIVE_LOOKAHEAD,
    RE_EXEC_STATE_GREEDY_QUANT,
} REExecStateEnum;

typedef struct REExecState {
    REExecStateEnum type : 8;
    uint8_t stack_len;
    int count; /* only used for RE_EXEC_STATE_GREEDY_QUANT */
    int cptr;
    int pc;
    int buf[0];
} REExecState;

typedef struct {
    int cbuf; /// const uint8_t *cbuf;
    int cbuf_end; /// const uint8_t *cbuf_end;
    /* 0 = 8 bit chars, 1 = 16 bit chars, 2 = 16 bit chars, UTF-16 */
    int cbuf_type; 
    int capture_count;
    int stack_size_max;
    BOOL multi_line;
    BOOL ignore_case;
    BOOL is_utf16;
    int opaque; /* used for stack overflow check */
    int state_size;
    int state_stack; /// uint8_t *state_stack;
    int state_stack_size;
    int state_stack_len;
} REExecContext;

#endif // LRE_EXEC_BACKTRRACK_H