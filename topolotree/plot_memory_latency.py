# See https://stackoverflow.com/a/19521297/3187068
import matplotlib
matplotlib.use('pdf')
font = {'size': 16}
matplotlib.rc('font', **font)

from typing import Any, List
import argparse
import matplotlib.pyplot as plt
import numpy as np
import os
import pandas as pd
import re

markers = ["o", "^", "s", "D", "p", "P", "X", "d"]

def plot_lt(rows: pd.DataFrame, ax: plt.Axes, marker: str, label: str, scale: int) -> None:
    # throughput_std = rows["std"] / scale
    # throughput_mean = rows["mean"] / scale
    throughput_med = rows["percentile_50"] / scale
    throughput_low = rows["percentile_25"] / scale
    throughput_high = rows["percentile_75"] / scale
    cluster_depth = [2 ** v for v in rows["tree_depth"]]
    line = ax.plot(cluster_depth, throughput_med, marker, label=label, linewidth=2)[0]
    ax.fill_between(cluster_depth,
                    throughput_low,
                    throughput_high,
                    color = line.get_color(),
                    alpha=0.25)


def plot_lt_min(rows: pd.DataFrame, ax: plt.Axes, marker: str, label: str, scale: int) -> None:
    throughput_med = rows["min"] / scale
    cluster_depth = rows["tree_depth"]
    line = ax.plot(cluster_depth, throughput_med, marker, label=label, linewidth=2)[0]

def main(args) -> None:
    fig, ax = plt.subplots(1, 1, figsize=(8, 4))
    # ax.set_yscale('log')

    if args.key == "memory_delta":
        ax.set_ylim(top=400)
    ax.set_xscale('log', base=2)
    
    dfs = pd.read_csv(args.results)


    # Abbreviate fields.
    for i, df in enumerate([dfs.groupby(["protocol"])]):
        for protocol, group in df:
            if protocol == "pn":
                protocol = "PN-Counter"
            elif protocol == "pn_delta":
                protocol = "\"Delta-PN\""
            elif protocol == "topolo":
                protocol = "OnceTree"
            if args.key == "memory_delta":
                plot_lt(group[group["kind"] == args.key], ax, markers[i] + "-", protocol, 1000 if args.key == 'latency' else 1000000)
            else:
                plot_lt(group[group["kind"] == args.key], ax, markers[i] + "-", protocol, 1000 if args.key == 'latency' else 1000000)

    ax.set_title('')
    ax.set_xlabel('# of nodes (log scale)')
    ax.set_ylabel('Latency (ms)' if args.key == 'latency' else 'Memory (MB)')
    ax.legend(loc='upper left')
    ax.grid()
    fig.savefig(args.output, bbox_inches='tight')
    print(f'Wrote plot to {args.output}.')

    fig_leg = plt.figure(figsize=(len(args.title)*3, 0.5))
    ax_leg = fig_leg.add_subplot(111)
    # add the legend from the previous axes
    ax_leg.legend(*ax.get_legend_handles_labels(), loc='center', ncol=len(args.title))
    # hide the axes frame and the x/y labels
    ax_leg.axis('off')
    # fig_leg.savefig('legend.pdf')


def get_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()

    parser.add_argument('--results',
                        type=argparse.FileType('r'),
                        help='results.csv file')
    parser.add_argument('--title',
                        type=str,
                        help='Title for each experiment')
    parser.add_argument('--key',
                        type=str,
                        help='Key')
    parser.add_argument('--output',
                        type=str,
                        default='compartmentalized_lt.pdf',
                        help='Output filename')

    return parser


if __name__ == '__main__':
    parser = get_parser()
    main(parser.parse_args())