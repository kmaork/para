import matplotlib.pyplot as plt
import re
import numpy as np
from collections import defaultdict
import matplotlib.ticker as mtick


def rolling(x, w):
    return np.convolve(x, np.ones(w) / w, mode='valid')


sep = lambda x: list(zip(*x))
max_list = defaultdict(int)
push_list = defaultdict(int)
pop_list = defaultdict(int)
arr = np.zeros((100, 100), dtype=int)

for line in open('log', 'r'):
    g = re.match(r'(?P<x>\d+), (?P<i>\d+), (?P<j>\d+)', line)
    max_sample = int(g['x'])
    push_sample = int(g['i'])
    pop_sample = int(g['j'])
    max_list[max_sample] += 1
    push_list[push_sample] += 1
    pop_list[pop_sample] += 1
    arr[push_sample][pop_sample] += 1


def graph():
    for d in [max_list, push_list, pop_list]:
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


def heatmap():
    plt.imshow(arr, cmap='hot', interpolation='nearest', origin='lower', vmax=arr.max())
    plt.xlabel("Pushes")
    plt.ylabel("Pops")
    plt.show()


heatmap()
