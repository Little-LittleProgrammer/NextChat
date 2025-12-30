 // 检测是否在 Tauri 环境中运行
export function isTauriEnv(): boolean {
    if (typeof window === "undefined") {
      return false;
    }
    // @ts-ignore
    return window.__TAURI__ !== undefined;
  }
  
  // 根据环境动态导入正确的 actions 模块
  export async function getMcpActions() {
    if (isTauriEnv()) {
      return await import("./actions.tauri");
    } else {
      return await import("./actions");
    }
  }
  