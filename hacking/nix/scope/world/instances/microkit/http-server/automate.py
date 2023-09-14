import sys
import time
import argparse
import pexpect
from requests import Session

HTTP_URL_BASE = 'http://localhost:8080'
HTTPS_URL_BASE = 'https://localhost:8443'

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
    child.expect('completed system invocations', timeout=5)

    time.sleep(3)

    for url_base in [HTTP_URL_BASE, HTTPS_URL_BASE]:
        sess = Session()
        url = url_base + '/About/'
        r = sess.get(url, verify=False, timeout=5)
        print(r.status_code)
        r.raise_for_status()

if __name__ == '__main__':
    main()
