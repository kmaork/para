import matplotlib.pyplot as plt
import re
import numpy as np

MAX = 200

def rolling(x, w):
    return np.convolve(x, np.ones(w) / w, mode='valid')


max_list = []
push_list = []
pop_list = []

for f in ['log200', 'log200.2', 'log200.3', 'log200.4']:
    for line in open(f, 'r'):
        g = re.match(r'(?P<max>\d+), (?P<push>\d+), (?P<pop>\d+)', line)
        max_list.append(int(g['max']))
        push_list.append(int(g['push']))
        pop_list.append(int(g['pop']))


def heatmap():
    arr = np.zeros((MAX, MAX), dtype=int)
    for push, pop in zip(push_list, pop_list):
        arr[push][pop] += 1
    plt.imshow(arr, cmap='hot', origin='lower')
    plt.xlabel("Pushes")
    plt.ylabel("Pops")
    plt.show()

print(np.mean(push_list), np.mean(pop_list))
heatmap()
