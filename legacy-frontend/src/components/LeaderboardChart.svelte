<script lang="ts">
    import { onDestroy } from "svelte";
    import {
        Chart,
        CategoryScale,
        LinearScale,
        TimeScale,
        LineElement,
        LineController,
        PointElement,
        Title,
        Tooltip,
        Legend,
        type ChartConfiguration,
    } from "chart.js";
    import "chartjs-adapter-date-fns";
    import type { LeaderboardEntry, ScorePoint } from "../api";

    Chart.register(
        CategoryScale,
        LinearScale,
        TimeScale,
        LineElement,
        LineController,
        PointElement,
        Title,
        Tooltip,
        Legend,
    );

    interface Props {
        teams: LeaderboardEntry[];
        maxTeams?: number;
    }

    const { teams, maxTeams = 10 }: Props = $props();

    const colorPalette = [
        { main: "#ff6384", light: "#ff8fa3" },
        { main: "#36a2eb", light: "#6bb6ff" },
        { main: "#ffce56", light: "#ffd97d" },
        { main: "#4bc0c0", light: "#7dd3d3" },
        { main: "#9966ff", light: "#b794ff" },
        { main: "#ff9f40", light: "#ffb869" },
        { main: "#c9cbcf", light: "#dddfe4" },
    ];

    let canvas = $state<HTMLCanvasElement | undefined>(undefined);
    let chart = $state<Chart | null>(null);

    function findLatestTimestamp(teams: LeaderboardEntry[]): number {
        let latest = 0;
        teams.forEach((team) => {
            if (team.scoreHistory && team.scoreHistory.length > 0) {
                const teamLatest = Math.max(
                    ...team.scoreHistory.map((point) => {
                        const utcDate = new Date(point.date + "Z");
                        return utcDate.getTime();
                    }),
                );
                latest = Math.max(latest, teamLatest);
            }
        });
        return latest;
    }

    function transformScoreHistory(
        scoreHistory: ScorePoint[],
        latestTimestamp: number,
    ) {
        if (!scoreHistory || scoreHistory.length === 0) {
            return [];
        }

        let transformedData = scoreHistory.map((point) => {
            const utcDate = new Date(point.date + "Z");
            return {
                x: utcDate.getTime(),
                y: point.score,
            };
        });

        if (scoreHistory.length === 1) {
            const point = scoreHistory[0];
            const utcDate = new Date(point.date + "Z");
            const originalTime = utcDate.getTime();
            transformedData = [
                { x: originalTime, y: point.score },
                { x: originalTime + 1, y: point.score },
            ];
        }

        const lastPoint = transformedData[transformedData.length - 1];
        if (lastPoint.x < latestTimestamp) {
            transformedData.push({
                x: latestTimestamp,
                y: lastPoint.y,
            });
        }

        return transformedData;
    }

    $effect(() => {
        if (canvas && teams.length > 0) {
            if (!chart) {
                createChart();
            } else {
                updateChart();
            }
        } else if (chart) {
            chart.destroy();
            chart = null;
        }
    });

    onDestroy(() => {
        if (chart) {
            chart.destroy();
            chart = null;
        }
    });

    function createChart() {
        if (!canvas) return;
        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        const topTeams = teams.slice(0, maxTeams);
        const latestTimestamp = findLatestTimestamp(topTeams);
        const datasets = topTeams.map((team, index) =>
            createDataset(team, index, latestTimestamp),
        );

        const config: ChartConfiguration = {
            type: "line",
            data: { datasets },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        display: true,
                        position: "bottom",
                        labels: {
                            usePointStyle: true,
                            padding: 12,
                            font: {
                                family: "Smiley Sans",
                                size: 14,
                            },
                            color: "#111827",
                        },
                    },
                    tooltip: {
                        backgroundColor: "#e6e6d2",
                        titleColor: "#111827",
                        bodyColor: "#111827",
                        borderColor: "#111827",
                        borderWidth: 1,
                        padding: 12,
                        displayColors: true,
                        titleFont: { family: "Satoshi" },
                        bodyFont: { family: "Satoshi" },
                        callbacks: {
                            title: (context) => {
                                const timestamp = context[0].parsed.x;
                                return new Date(timestamp).toLocaleString();
                            },
                            label: (context) => {
                                return `${context.dataset.label}: ${context.parsed.y} points`;
                            },
                        },
                    },
                },
                scales: {
                    y: {
                        beginAtZero: true,
                        title: {
                            display: true,
                            text: "Score",
                            font: {
                                family: "Smiley Sans",
                                size: 20,
                                weight: "bold",
                            },
                            color: "#111827",
                        },
                        grid: { display: false },
                        ticks: {
                            stepSize: 100,
                            color: "#111827",
                            font: { family: "Satoshi" },
                        },
                    },
                    x: {
                        type: "time",
                        time: {
                            unit: "hour",
                            displayFormats: {
                                hour: "MMM dd HH:mm",
                                day: "MMM dd",
                            },
                        },
                        grid: { display: false },
                        ticks: {
                            color: "#111827",
                            font: { family: "Satoshi" },
                            maxTicksLimit: 10,
                        },
                    },
                },
                interaction: {
                    intersect: false,
                    mode: "index",
                },
                animation: {
                    duration: 1000,
                    easing: "easeInOutQuart",
                },
            },
        };

        chart = new Chart(ctx, config);
    }

    function createDataset(
        team: LeaderboardEntry,
        index: number,
        latestTimestamp: number,
    ) {
        const color = colorPalette[index % colorPalette.length];

        return {
            label: team.name,
            data: transformScoreHistory(
                team.scoreHistory || [],
                latestTimestamp,
            ),
            backgroundColor: color.light,
            borderColor: color.main,
            borderWidth: 2,
            pointRadius: 0,
            pointHoverRadius: 0,
            fill: false,
            tension: 0,
            order: index + 1,
            pointBorderColor: color.main,
            pointBackgroundColor: color.light,
        };
    }

    function updateChart() {
        if (!chart) return;
        const topTeams = teams.slice(0, maxTeams);
        const latestTimestamp = findLatestTimestamp(topTeams);
        chart.data.datasets = topTeams.map((team, index) =>
            createDataset(team, index, latestTimestamp),
        );
        chart.update("active");
    }
</script>

<div class="chart-container">
    {#if teams.length === 0}
        <div class="no-data">No score data available yet</div>
    {:else}
        <canvas bind:this={canvas}></canvas>
    {/if}
</div>

<style>
    .chart-container {
        position: relative;
        width: 100%;
        height: 500px;
        margin: 2rem 0;
        background-color: #e6e6d2;
        border-image: url("/border-big-off-white.png") 12 / 8px round;
        border-image-outset: 2px;
        padding: 1.5rem;
    }

    .no-data {
        display: flex;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: #666;
        font-family: "Satoshi", sans-serif;
    }

    @media (max-width: 768px) {
        .chart-container {
            height: 400px;
            margin: 1rem 0;
            padding: 1rem;
        }
    }
</style>
