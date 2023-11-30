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

def plot_lt(throughput_rows: pd.DataFrame, latency_rows: pd.DataFrame, ax: plt.Axes, marker: str, label: str) -> None:
    throughput = throughput_rows["mean"]# / 1000
    throughput_std = throughput_rows["std"]# / 1000
    latency = latency_rows["percentile_50"] / 1000
    line = ax.plot(throughput, latency, marker, label=label, linewidth=2)[0]
    ax.fill_betweenx(latency,
                     throughput - throughput_std,
                     throughput + throughput_std,
                     color = line.get_color(),
                     alpha=0.25)


def main(args) -> None:
    fig, ax = plt.subplots(1, 1, figsize=(8, 4))
    ax.set_ylim(top=25)
    ax.set_xscale('log')
    ax.get_xaxis().set_major_formatter(matplotlib.ticker.ScalarFormatter())
    
    dfs = pd.read_csv(args.results)

    # Abbreviate fields.
    for i, df in enumerate([dfs.groupby(["protocol"])]):
        for protocol, group in df:
            throughput_rows = group[group["kind"] == "total_throughput"]
            latency_rows = group[group["kind"] == "latency"]

            if protocol == "pn":
                protocol = "PN-Counter"
            elif protocol == "pn_delta":
                protocol = "\"Delta-PN\""
            elif protocol == "topolo":
                protocol = "OnceTree"

            plot_lt(throughput_rows, latency_rows, ax, markers[i] + "-", protocol)

    ax.set_title('')
    ax.set_xlabel('Throughput (ops / second)')
    ax.set_ylabel('Median Latency (ms)')
    ax.legend(loc='upper right')
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
    parser.add_argument('--output',
                        type=str,
                        default='compartmentalized_lt.pdf',
                        help='Output filename')

    return parser


if __name__ == '__main__':
    parser = get_parser()
    main(parser.parse_args())