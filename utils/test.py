#!/usr/bin/env python3

import re
import socket
import random
import threading
import time


class PlayerThread (threading.Thread):
    def __init__(self, name, num):
        super().__init__()
        self.name = name
        self.num = num

    def run(self):
        s = socket.socket()
        s.connect(('localhost', 2342))
        buffer_size = 4096

        try:
            time.sleep(self.num)
            s.send(('player name %s\n' % self.name).encode())
            s.send(b'table list\n')
            data = s.recv(buffer_size).decode()
            print(data, end='')
            match = re.search('\t([^\s]+) ', data)
            if match is None:
                s.send(b'table new Kuchen\n')
                s.send(b'table list\n')
                data = s.recv(buffer_size).decode()
                match = re.search('\t([^\s]+) ', data)
            tablehash = match.group(1)
            s.send(('table join %s\n' % tablehash).encode())
            if self.num == 0:
                time.sleep(3)
                print('')
                print('------STARTER------')
                s.send(b'player state\n')
                print(s.recv(buffer_size).decode(), end='')
                s.send(b'game start\n')
                print(s.recv(buffer_size).decode(), end='')
                s.send(b'game state\n')
                data = s.recv(buffer_size).decode()
                card = re.match('cards (..)', data).group(1)
                s.send(('game put %s\n' % card).encode())
                print(s.recv(buffer_size).decode(), end='')
                s.send(('game put %s 0\n' % card).encode())
                print(s.recv(buffer_size).decode(), end='')
                s.send(b'game state\n')
                print(s.recv(buffer_size).decode(), end='')
            else:
                time.sleep(6)
                time.sleep(self.num)
                print('')
                print('---------%1d---------' % self.num)
                s.send(b'player state\n')
                print(s.recv(buffer_size).decode(), end='')
                s.send(b'game state\n')
                data = s.recv(buffer_size).decode()
                match = re.match('cards (..)', data)
                if match is None:
                    data = s.recv(buffer_size).decode()
                    match = re.match('cards (..)', data)
                card = match.group(1)
                s.send(('game put %s\n' % card).encode())
                print(s.recv(buffer_size).decode(), end='')
                s.send(('game put %s 0\n' % card).encode())
                print(s.recv(buffer_size).decode(), end='')
                s.send(b'game state\n')
                print(s.recv(buffer_size).decode(), end='')
        except Exception as e:
            print(e)
        finally:
            s.close()


threads = []

for (num, name) in enumerate(['Kekse', 'Kuchen', 'Quark']):
    p = PlayerThread(name, num)
    p.start()
    threads.append(p)

for t in threads:
    t.join()
