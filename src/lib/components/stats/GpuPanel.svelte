<script lang="ts">
  import { faMicrochip } from "@fortawesome/free-solid-svg-icons";
  import { PanelHeader, ProgressBar, StatItem } from "$lib/components";
  import { formatBytes, formatPercentage } from "$lib/utils";

  export let gpuStats:
    | {
        name: string;
        utilization: number;
        memory_used: number;
        memory_total: number;
        temperature: number;
        power_usage: number;
      }[]
    | null;

  $: averageUsage = formatPercentage(
    gpuStats.map((x) => x.utilization).reduce((a, b) => a + b, 0) /
      gpuStats.length,
  );
</script>

<div class="stat-panel">
  <PanelHeader icon={faMicrochip} title="GPU" usageValue={averageUsage} />
  {#if gpuStats && gpuStats.length > 0}
    {#each gpuStats as gpu}
      <div class="gpu-stats">
        <span>{gpu.name}</span>
        <div class="stat-item with-progress">
          <ProgressBar
            label="GPU Usage"
            value={gpu.utilization}
            labelWidth="5rem"
            valueWidth="2.5rem"
          />
        </div>
        <StatItem
          label="Memory"
          value={`${formatBytes(gpu.memory_used)} / ${formatBytes(gpu.memory_total)}`}
        />
        <StatItem label="Temperature" value={`${gpu.temperature}°C`} />
        <StatItem label="Power" value={`${gpu.power_usage / 1000}W`} />
      </div>
    {/each}
  {:else}
    <div class="no-gpu">No NVIDIA GPU detected</div>
  {/if}
</div>

<style>
  .stat-panel {
    flex: 1.5;
    min-width: 0;
    background-color: var(--mantle);
    border-radius: 6px;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
  }

  .gpu-stats {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .no-gpu {
    color: var(--subtext0);
    text-align: center;
    padding: 1rem;
  }

  span {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.7rem;
    line-height: 1.2;
    margin: 0;
    padding: 0;
    color: var(--subtext0);
  }

  h4 {
    margin: 0;
    color: var(--text);
    font-size: 0.9rem;
  }
</style>
