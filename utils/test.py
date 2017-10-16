#!/usr/bin/env python3

import re
import socket
import random
import threading
import time
import traceback
import sys


class PlayerThread (threading.Thread):
    def __init__(self, name, num):
        super().__init__()
        self.name = name
        self.num = num

    def send(self, msg, print_all=False, hide_err=False):
        buffer_size = 4096
        try:
            self.socket.recv(buffer_size)
        except socket.timeout:
            pass
        self.socket.send((msg + '\n').encode())
        try:
            answer = self.socket.recv(buffer_size).decode()
            if print_all or (answer.startswith('ERROR') and not hide_err):
                print(answer, end='')
            if answer.startswith('ERROR'):
                return None
            return answer
        except socket.timeout:
            return None

    def run(self):
        self.socket = socket.socket()
        self.socket.connect(('localhost', 2342))
        self.socket.settimeout(0.1)

        try:
            time.sleep(self.num)
            self.send('player name %s' % self.name)
            data = self.send('table list')
            match = re.search('\t([^\s]+) ', data)
            if match is None:
                self.send('table new Kuchen')
                data = self.send('table list')
                match = re.search('\t([^\s]+) ', data)
            tablehash = match.group(1)
            self.send('table join %s' % tablehash)
            if self.num == 0:
                time.sleep(3)
                print('')
                print('------STARTER------')
                self.send('game start')
                self.send('player state', print_all=True)
                self.send('game state', print_all=True)
                for i in range(5):
                    data = None
                    while data is None:
                        data = self.send('game state')
                    card = re.match('cards (..)', data).group(1)
                    self.send('game put %s' % card, hide_err=True)
                    for j in range(5):
                        self.send('game put %s %d' % (card, j), hide_err=True)
                self.send('game state', print_all=True)
                time.sleep(15)
                self.send('quit')
            else:
                time.sleep(6)
                time.sleep(2*self.num)
                print('')
                print('---------%1d---------' % self.num)
                self.send('player state', print_all=True)
                self.send('game state', print_all=True)
                for i in range(5):
                    data = None
                    while data is None:
                        data = self.send('game state')
                    match = re.match('cards (..)', data)
                    if match is None:
                        data = self.send('game state')
                        match = re.match('cards (..)', data)
                    card = match.group(1)
                    self.send('game put %s' % card, hide_err=True)
                    for j in range(5):
                        self.send('game put %s %d' % (card, j), hide_err=True)
                self.send('game state', print_all=True)
        except Exception:
            traceback.print_exc(file=sys.stderr)
        finally:
            self.socket.close()


threads = []

for (num, name) in enumerate(['Kekse', 'Kuchen', 'Quark', 'Ente']):
    p = PlayerThread(name, num)
    p.start()
    threads.append(p)

for t in threads:
    t.join()
