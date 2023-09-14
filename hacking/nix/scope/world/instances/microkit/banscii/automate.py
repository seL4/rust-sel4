import sys
import argparse
import pexpect

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('simulate')
    args = parser.parse_args()
    run(args)

def run(args):
    child = pexpect.spawn(args.simulate, encoding='utf-8')
    child.logfile = sys.stdout
    child.expect('banscii>', timeout=3)
    child.sendline('Hello, World!')
    child.expect('banscii>', timeout=1)
    print()

if __name__ == '__main__':
    main()
