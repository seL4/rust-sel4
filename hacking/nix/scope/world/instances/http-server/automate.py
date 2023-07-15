import sys
import time
import argparse
import pexpect
from requests import Session

URL_BASE = 'http://localhost:8000'

def mk_url(path):
    return URL_BASE + path

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('simulate')
    args = parser.parse_args()
    run(args)

def run(args):
    child = pexpect.spawn(args.simulate, encoding='utf-8')
    child.logfile = sys.stdout
    child.expect('CapDL initializer done, suspending', timeout=5)
    # child.interact()

    time.sleep(3)

    sess = Session()
    r = sess.get(mk_url('/About/'), timeout=5)
    print(r.status_code)
    r.raise_for_status()

if __name__ == '__main__':
    main()
