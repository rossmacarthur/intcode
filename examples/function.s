; This example demonstrates how you can implement a stack and function call. The
; `_main` routine calls `function` with two parameters and outputs the return
; value.

    ARB #end+10           ; allocate 10 addresses for the stack

_main:
    ADD  #72,    #0, rb-1 ; push the first parameter to the stack
    ADD #105,    #0, rb-2 ; push the second parameter to the stack
    ADD #return, #0, rb-3 ; push the return address to the stack
    ARB #-3
    JZ  #0, #function     ; call `function`
  return:
    OUT rb-4              ; output the value returned by `function`

    HLT                   ; end the program

function:
    ARB #-1               ; reserve stack space for one local variable
    OUT rb+3              ; output the first parameter
    OUT rb+2              ; output the second parameter
    ADD #10, #0, rb       ; the local variable will be the return value
    ARB #1                ; remove the local variable from the stack
    ARB #3                ; remove the two parameters and return from the stack
    JZ  #0, rb-3          ; jump to the return address

end:
    DB 0
