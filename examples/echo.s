; This example demonstrates how you can implement a pointer to a string. An
; input is read into `message` and then immediately output.

    ADD #message, #-1, ptr ; set `ptr` to the beginning of `message`
input:
    ADD ptr, #1, ptr       ; increment the `ptr` value
    ADD ptr, #0, ip+1      ; set the first parameter of `IN` to the `ptr` value
    IN  _                  ; read the next character
    ADD ptr, #0, ip+1      ; set the first parameter of `EQ` to the `ptr` value
    EQ  _, #10, ip+1       ; check whether the character is equal to a newline
    JZ  #_, #input         ; if the next character is not-equal, then loop

    ADD #message, #0, ptr  ; set `ptr` to the beginning of `message`
output:
    ADD ptr, #0, ip+1      ; set the first parameter of `OUT` to the `ptr` value
    OUT _                  ; output the next character
    ADD ptr, #1, ptr       ; increment the `ptr` value
    ADD ptr, #0, ip+1      ; set the first parameter of `JNZ` to the `ptr` value
    JNZ _, #output         ; if the next character is non-zero, then loop

    HLT

ptr:
    DB 0
message:
    DB 0
