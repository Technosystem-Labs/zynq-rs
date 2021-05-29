from time import sleep
from pyftdi.ftdi import Ftdi

POR = 1 << 7

def main():
    dev = Ftdi()
    dev.open_bitbang_from_url("ftdi://ftdi:4232h/0")
    dev.set_bitmode(POR, Ftdi.BitMode.BITBANG)
    dev.write_data(bytes([0]))
    sleep(0.1)
    dev.write_data(bytes([POR]))
    sleep(0.1)
    dev.close()

if __name__ == "__main__":
    main()
