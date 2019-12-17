.section ".text.exceptions"
.global exception_vector

exception_vector:
    b Reset
    b UndefinedInstruction
    b SoftwareInterrupt
    b PrefetchAbort
    b DataAbort
    b ReservedException
    b IRQ
    b FIQ
