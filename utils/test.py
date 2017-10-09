#!/usr/bin/env python3

import re
import socket

s = socket.socket()
s.connect(('localhost', 2342))

try:
    s.send(b'player name Kekse\n')
    s.send(b'table new Kuchen\n')
    s.send(b'table list\n')
    data = s.recv(1024).decode()
    tablehash = re.search('\t([^\s]+) ', data).group(1)
    s.send(('table join %s\n' % tablehash).encode())
    s.send(b'game start\n')
    s.send(b'table list\n')
    print(s.recv(1024).decode(), end='')
except Exception as e:
    print(e)
finally:
    s.close()
