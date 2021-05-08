import matplotlib.pyplot as plt
import re
import numpy as np
from collections import defaultdict
import matplotlib.ticker as mtick

def rolling(x, w):
    return np.convolve(x, np.ones(w) / w, mode='valid')


sep = lambda x: list(zip(*x))
x = defaultdict(int)
i = defaultdict(int)
j = defaultdict(int)

for line in open('log', 'r'):
    g = re.match(r'(?P<x>\d+), (?P<i>\d+), (?P<j>\d+)', line)
    x[int(g['x'])] += 1
    i[int(g['i'])] += 1
    j[int(g['j'])] += 1

for d in [x, i, j]:
    w = 5
    x, y = sep(sorted(d.items()))
    # plt.plot(np.fft.fft(y))
    if w:
        y = [val * 100 / sum(y) for val in rolling(y, w)]
        x = x[w // 2:-w // 2 + 1]
    plt.plot(x, y)

plt.legend(["Max", "Pushes", "Pops"])
plt.xlabel("Num")
plt.gca().yaxis.set_major_formatter(mtick.PercentFormatter())
plt.show()


