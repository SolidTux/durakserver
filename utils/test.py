#!/usr/bin/env python3

import re
import socket
import random
import threading
import time


class PlayerThread (threading.Thread):
    def __init__(self, name):
        super().__init__()
        self.name = name

    def run(self):
        s = socket.socket()
        s.connect(('localhost', 2342))
        time.sleep(random.random())

        starter = False
        try:
            s.send(('player name %s\n' % self.name).encode())
            s.send(b'table list\n')
            data = s.recv(1024).decode()
            match = re.search('\t([^\s]+) ', data)
            if match is None:
                s.send(b'table new Kuchen\n')
                s.send(b'table list\n')
                data = s.recv(1024).decode()
                match = re.search('\t([^\s]+) ', data)
                starter = True
            tablehash = match.group(1)
            s.send(('table join %s\n' % tablehash).encode())
            if starter:
                time.sleep(3)
                s.send(b'game start\n')
                print(s.recv(1024).decode(), end='')
                time.sleep(3)
                s.send(b'table list\n')
                print(s.recv(1024).decode(), end='')
        except Exception as e:
            print(e)
        finally:
            s.close()


threads = []

for name in ['Kekse', 'Kuchen', 'Quark']:
    p = PlayerThread(name)
    p.start()
    threads.append(p)

for t in threads:
    t.join()
