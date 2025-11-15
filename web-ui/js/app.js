// Orban Agent Web UI - 主應用程式
// 模擬數據展示（實際應該從 Rust 後端 API 獲取）

class OrbanAgentUI {
    constructor() {
        this.startTime = new Date();
        this.mockData = this.generateMockData();
        this.init();
    }

    // 初始化
    init() {
        this.updateEarningsOverview();
        this.updateGPUList();
        this.updateEarningsHistory();
        this.updateAgentInfo();

        // 每 5 秒更新一次 GPU 狀態
        setInterval(() => this.updateGPUStatus(), 5000);

        // 每秒更新運行時長
        setInterval(() => this.updateUptime(), 1000);

        // 每 10 秒模擬新的收益記錄（展示用）
        setInterval(() => this.addMockEarning(), 10000);
    }

    // 生成模擬數據
    generateMockData() {
        return {
            agent: {
                id: 'agent-' + Math.random().toString(36).substr(2, 9),
                version: '0.1.0',
                isRunning: true,
                tasksCompleted: 127,
                tasksFailed: 3
            },
            gpus: [
                {
                    name: 'NVIDIA GeForce RTX 4090',
                    type: 'NVIDIA',
                    utilization: 0.65,
                    memoryUsed: 18.2,
                    memoryTotal: 24.0,
                    temperature: 72.5,
                    powerUsage: 320.5
                },
                {
                    name: 'NVIDIA GeForce RTX 3090',
                    type: 'NVIDIA',
                    utilization: 0.45,
                    memoryUsed: 12.8,
                    memoryTotal: 24.0,
                    temperature: 68.0,
                    powerUsage: 280.0
                }
            ],
            earnings: {
                total: 8.45,      // 累計約 2 週的收益
                today: 0.58,      // RTX 4090 跑一天的實際收益
                pending: 0.15,    // 待確認的少量收益
                history: this.generateMockHistory()
            }
        };
    }

    // 生成模擬歷史記錄
    generateMockHistory() {
        const history = [];
        const gpuModels = ['RTX 4090', 'RTX 3090'];
        const now = new Date();

        for (let i = 0; i < 10; i++) {
            const timestamp = new Date(now - i * 3600000); // 每小時一筆
            const gpuModel = gpuModels[Math.floor(Math.random() * gpuModels.length)];
            const gpuHours = (Math.random() * 2 + 0.5).toFixed(2);
            const rate = gpuModel.includes('4090') ? 0.025 : 0.018;
            const amount = (gpuHours * rate).toFixed(3);

            history.push({
                timestamp,
                gpuModel,
                gpuHours,
                rate,
                amount,
                status: Math.random() > 0.3 ? 'confirmed' : 'pending'
            });
        }

        return history;
    }

    // 更新收益概覽
    updateEarningsOverview() {
        document.getElementById('totalEarnings').textContent =
            `$${this.mockData.earnings.total.toFixed(2)}`;
        document.getElementById('todayEarnings').textContent =
            `$${this.mockData.earnings.today.toFixed(2)}`;
        document.getElementById('pendingEarnings').textContent =
            `$${this.mockData.earnings.pending.toFixed(2)}`;
        document.getElementById('tasksCompleted').textContent =
            this.mockData.agent.tasksCompleted;
    }

    // 更新 GPU 列表
    updateGPUList() {
        const gpuList = document.getElementById('gpuList');
        gpuList.innerHTML = '';

        this.mockData.gpus.forEach((gpu, index) => {
            const gpuCard = this.createGPUCard(gpu, index);
            gpuList.appendChild(gpuCard);
        });
    }

    // 創建 GPU 卡片
    createGPUCard(gpu, index) {
        const card = document.createElement('div');
        card.className = 'gpu-card';
        card.innerHTML = `
            <div class="gpu-header">
                <div class="gpu-name">GPU ${index}: ${gpu.name}</div>
                <div class="gpu-type">${gpu.type}</div>
            </div>

            <div class="gpu-stat">
                <span class="stat-label">使用率</span>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: ${gpu.utilization * 100}%"></div>
                </div>
                <span class="stat-value">${(gpu.utilization * 100).toFixed(1)}%</span>
            </div>

            <div class="gpu-stat">
                <span class="stat-label">記憶體使用</span>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: ${(gpu.memoryUsed / gpu.memoryTotal) * 100}%"></div>
                </div>
                <span class="stat-value">${gpu.memoryUsed.toFixed(1)} / ${gpu.memoryTotal.toFixed(1)} GB</span>
            </div>

            <div class="gpu-stat">
                <span class="stat-label">溫度</span>
                <span class="stat-value">${gpu.temperature.toFixed(1)}°C</span>
            </div>

            <div class="gpu-stat">
                <span class="stat-label">功耗</span>
                <span class="stat-value">${gpu.powerUsage.toFixed(1)}W</span>
            </div>
        `;
        return card;
    }

    // 更新 GPU 狀態（模擬變化）
    updateGPUStatus() {
        this.mockData.gpus.forEach(gpu => {
            // 模擬使用率變化
            gpu.utilization = Math.max(0.1, Math.min(0.95,
                gpu.utilization + (Math.random() - 0.5) * 0.1));

            // 模擬溫度變化
            gpu.temperature = Math.max(50, Math.min(85,
                gpu.temperature + (Math.random() - 0.5) * 3));

            // 模擬功耗變化
            gpu.powerUsage = Math.max(100, Math.min(400,
                gpu.powerUsage + (Math.random() - 0.5) * 20));
        });

        this.updateGPUList();
    }

    // 更新收益歷史
    updateEarningsHistory() {
        const tbody = document.getElementById('earningsHistory');
        tbody.innerHTML = '';

        this.mockData.earnings.history.forEach(record => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>${this.formatDate(record.timestamp)}</td>
                <td>${record.gpuModel}</td>
                <td>${record.gpuHours}h</td>
                <td>$${record.rate.toFixed(3)}/h</td>
                <td>$${record.amount}</td>
                <td class="status-${record.status}">
                    ${record.status === 'confirmed' ? '已確認' : '待確認'}
                </td>
            `;
            tbody.appendChild(row);
        });
    }

    // 添加新的模擬收益記錄（展示用）
    addMockEarning() {
        const gpuModel = this.mockData.gpus[
            Math.floor(Math.random() * this.mockData.gpus.length)
        ].name.split(' ').slice(-2).join(' ');

        // 每次執行 0.05-0.2 小時（3-12 分鐘），更符合實際任務時長
        const gpuHours = (Math.random() * 0.15 + 0.05).toFixed(2);
        const rate = gpuModel.includes('4090') ? 0.025 : 0.018;
        const amount = (gpuHours * rate).toFixed(4);  // 顯示到小數點後 4 位

        const newRecord = {
            timestamp: new Date(),
            gpuModel,
            gpuHours,
            rate,
            amount,
            status: 'pending'
        };

        // 添加到歷史記錄最前面
        this.mockData.earnings.history.unshift(newRecord);

        // 只保留最新 10 筆
        if (this.mockData.earnings.history.length > 10) {
            this.mockData.earnings.history.pop();
        }

        // 更新總收益
        this.mockData.earnings.total += parseFloat(amount);
        this.mockData.earnings.today += parseFloat(amount);
        this.mockData.earnings.pending += parseFloat(amount);
        this.mockData.agent.tasksCompleted++;

        // 更新顯示
        this.updateEarningsOverview();
        this.updateEarningsHistory();

        // 顯示通知效果
        this.showNotification(`新收益: $${amount} (${gpuModel})`);
    }

    // 更新 Agent 資訊
    updateAgentInfo() {
        document.getElementById('agentId').textContent = this.mockData.agent.id;
    }

    // 更新運行時長
    updateUptime() {
        const now = new Date();
        const diff = now - this.startTime;

        const hours = Math.floor(diff / 3600000);
        const minutes = Math.floor((diff % 3600000) / 60000);
        const seconds = Math.floor((diff % 60000) / 1000);

        document.getElementById('uptime').textContent =
            `${hours}h ${minutes}m ${seconds}s`;
    }

    // 格式化日期
    formatDate(date) {
        const month = String(date.getMonth() + 1).padStart(2, '0');
        const day = String(date.getDate()).padStart(2, '0');
        const hours = String(date.getHours()).padStart(2, '0');
        const minutes = String(date.getMinutes()).padStart(2, '0');

        return `${month}-${day} ${hours}:${minutes}`;
    }

    // 顯示通知
    showNotification(message) {
        // 可以在這裡添加更複雜的通知 UI
        console.log('📢', message);
    }
}

// 當 DOM 載入完成後啟動應用
document.addEventListener('DOMContentLoaded', () => {
    console.log('🚀 Orban Agent Web UI 啟動中...');
    const app = new OrbanAgentUI();
    console.log('✓ 應用初始化完成');
    console.log('📊 模擬數據已載入 - 這是展示用的前端界面');
    console.log('💡 提示：GPU 數據每 5 秒更新，收益記錄每 10 秒新增');
});
