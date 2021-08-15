    ARB #message  ; move the relative base to the beginning of our message

loop:
    OUT rb        ; output the current character in the message
    ARB #1        ; move the relative base to the next character
    JNZ rb, #loop ; if the next character is non-zero then go back to `loop`
    HLT

message:
    DB "Hello World!\n"
