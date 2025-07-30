// UART Register Offsets (NS16550A compatible)
.equ UART_BASE,     0x9000000       // Base address for UART device
.equ UART_RBR_THR,  0x00            // Receiver Buffer/Transmitter Holding Register
.equ UART_IER,      0x01            // Interrupt Enable Register  
.equ UART_IIR_FCR,  0x02            // Interrupt Identification/FIFO Control Register
.equ UART_LCR,      0x03            // Line Control Register
.equ UART_MCR,      0x04            // Modem Control Register
.equ UART_LSR,      0x05            // Line Status Register
.equ UART_MSR,      0x06            // Modem Status Register
.equ UART_SCR,      0x07            // Scratch Register

start:
    // Initialize UART base address
    movz x19, #0x0000, lsl #0
    movk x19, #0x0900, lsl #16       // x19 = 0x09000000
    
    // Initialize UART - inline without function calls
    mov w0, #0x07                    // Enable FIFO and clear FIFOs
    strb w0, [x19, #UART_IIR_FCR]
    
    mov w0, #0x03                    // 8-bit data
    strb w0, [x19, #UART_LCR]
    
    mov w0, #0x00                    // Disable interrupts
    strb w0, [x19, #UART_IER]
    
    mov w0, #0x00                    // No modem control
    strb w0, [x19, #UART_MCR]
    
    // Send 'H' character - inline
    mov w1, #'H'
    mov w2, #1000                    // Delay counter
delay1:
    subs w2, w2, #1
    b.ne delay1
    strb w1, [x19, #UART_RBR_THR]
    
    // Send 'E' character - inline
    mov w1, #'E'
    mov w2, #1000
delay2:
    subs w2, w2, #1
    b.ne delay2
    strb w1, [x19, #UART_RBR_THR]
    
    // Send 'L' character - inline
    mov w1, #'L'
    mov w2, #1000
delay3:
    subs w2, w2, #1
    b.ne delay3
    strb w1, [x19, #UART_RBR_THR]
    
    // Send 'L' character - inline
    mov w1, #'L'
    mov w2, #1000
delay4:
    subs w2, w2, #1
    b.ne delay4
    strb w1, [x19, #UART_RBR_THR]
    
    // Send 'O' character - inline
    mov w1, #'O'
    mov w2, #1000
delay5:
    subs w2, w2, #1
    b.ne delay5
    strb w1, [x19, #UART_RBR_THR]
    
    // Send newline - inline
    mov w1, #'\n'
    mov w2, #1000
delay6:
    subs w2, w2, #1
    b.ne delay6
    strb w1, [x19, #UART_RBR_THR]
    
    // Test register read/write - inline
    mov w0, #0xAA
    strb w0, [x19, #UART_SCR]        // Write to scratch register
    ldrb w1, [x19, #UART_SCR]       // Read back
    cmp w0, w1
    b.ne test_fail
    
    // Send 'P' for PASS
    mov w1, #'P'
    mov w2, #1000
delay7:
    subs w2, w2, #1
    b.ne delay7
    strb w1, [x19, #UART_RBR_THR]
    
    // Send 'A'
    mov w1, #'A'
    mov w2, #1000
delay8:
    subs w2, w2, #1
    b.ne delay8
    strb w1, [x19, #UART_RBR_THR]
    
    // Send 'S'
    mov w1, #'S'
    mov w2, #1000
delay9:
    subs w2, w2, #1
    b.ne delay9
    strb w1, [x19, #UART_RBR_THR]
    
    // Send 'S'
    mov w1, #'S'
    mov w2, #1000
delay10:
    subs w2, w2, #1
    b.ne delay10
    strb w1, [x19, #UART_RBR_THR]
    
    b test_done
    
test_fail:
    // Send 'F' for FAIL
    mov w1, #'F'
    mov w2, #1000
delay_fail1:
    subs w2, w2, #1
    b.ne delay_fail1
    strb w1, [x19, #UART_RBR_THR]
    
    // Send 'A'
    mov w1, #'A'
    mov w2, #1000
delay_fail2:
    subs w2, w2, #1
    b.ne delay_fail2
    strb w1, [x19, #UART_RBR_THR]
    
    // Send 'I'
    mov w1, #'I'
    mov w2, #1000
delay_fail3:
    subs w2, w2, #1
    b.ne delay_fail3
    strb w1, [x19, #UART_RBR_THR]
    
    // Send 'L'
    mov w1, #'L'
    mov w2, #1000
delay_fail4:
    subs w2, w2, #1
    b.ne delay_fail4
    strb w1, [x19, #UART_RBR_THR]
    
test_done:
    // Send final newline
    mov w1, #'\n'
    mov w2, #1000
delay_final:
    subs w2, w2, #1
    b.ne delay_final
    strb w1, [x19, #UART_RBR_THR]
    
    // Send digits 0-7 for FIFO test
    mov w20, #0                      // Counter
    
fifo_loop:
    cmp w20, #8
    b.ge fifo_done
    
    add w1, w20, #'0'                // Convert to ASCII
    mov w2, #2000                    // Longer delay
fifo_delay:
    subs w2, w2, #1
    b.ne fifo_delay
    
    strb w1, [x19, #UART_RBR_THR]
    add w20, w20, #1
    b fifo_loop
    
fifo_done:
    // Final newline
    mov w1, #'\n'
    mov w2, #1000
final_delay:
    subs w2, w2, #1
    b.ne final_delay
    strb w1, [x19, #UART_RBR_THR]
    
    // Exit to hypervisor - this should work now
    mov x0, #0                       // Exit code
    hvc #0                           // Hypervisor call to exit
