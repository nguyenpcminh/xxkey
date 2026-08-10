#ifndef BridgeHeader_h
#define BridgeHeader_h

#include <stdint.h>
#include <stdbool.h>

// Forward declarations matching Rust FFI
typedef struct Engine Engine;
typedef struct HookState HookState;

Engine* vietime_new_engine(void);
void vietime_free_engine(Engine* engine);
void vietime_reset_engine(Engine* engine);
void vietime_set_input_type(Engine* engine, uint8_t input_type);
void vietime_set_modern_orthography(Engine* engine, uint8_t modern);

const HookState* vietime_handle_key(
    Engine* engine,
    uint8_t event,
    uint8_t state,
    uint16_t data,
    uint8_t caps_status,
    bool other_control_key
);

uint16_t vietime_key_code_to_char(uint32_t key_code);

uint8_t vietime_get_hook_state_code(const HookState* state);
uint8_t vietime_get_hook_state_backspace_count(const HookState* state);
uint8_t vietime_get_hook_state_new_char_count(const HookState* state);
uint8_t vietime_get_hook_state_ext_code(const HookState* state);
uint32_t vietime_get_hook_state_char_at(const HookState* state, uint32_t index);

#endif /* BridgeHeader_h */
