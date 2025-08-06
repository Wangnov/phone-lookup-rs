import { invoke } from '@tauri-apps/api/core'

// 手机号查询结果类型定义（与后端PhoneNoInfo匹配）
export interface PhoneInfo {
  province: string
  city: string
  zip_code: string  // 后端是zip_code，不是zip
  area_code: string
  card_type: string
}

// 单个查询结果类型
export interface QueryResult {
  success: boolean
  data?: PhoneInfo
  error?: string
}

// 批量查询结果项类型（与后端BatchQueryResult匹配）
export interface BatchQueryItem {
  phone: string
  index: number
  result?: PhoneInfo  // 后端使用的是result字段，不是data
  error?: string
  // 为了向后兼容，添加computed properties
  success?: boolean
  data?: PhoneInfo
}

// 批量查询结果类型
export interface BatchQueryResult {
  success: boolean
  total: number
  successful: number
  failed: number
  results: BatchQueryItem[]
}

// 数据库信息类型
export interface DatabaseInfo {
  total_records: number
  cache_size: number
  cache_hits: number
  total_queries: number
}

// 应用信息类型
export interface AppInfo {
  name: string
  version: string
  author: string
  database_info: DatabaseInfo
}

// Tauri API 封装类
export class TauriAPI {
  /**
   * 查询单个手机号归属地信息
   * @param phone 手机号码
   * @returns 查询结果
   */
  static async queryPhone(phone: string): Promise<QueryResult> {
    try {
      const result = await invoke<PhoneInfo>('query_phone', { phone })
      return {
        success: true,
        data: result
      }
    } catch (error) {
      console.error('查询手机号失败:', error)
      return {
        success: false,
        error: error instanceof Error ? error.message : '查询失败'
      }
    }
  }

  /**
   * 批量查询手机号归属地信息
   * @param phones 手机号码数组
   * @returns 批量查询结果数组
   */
  static async queryPhonesBatch(phones: string[]): Promise<BatchQueryItem[]> {
    try {
      const results = await invoke<BatchQueryItem[]>('query_phones_batch', { phones })
      return results
    } catch (error) {
      console.error('批量查询失败:', error)
      return phones.map((phone, index) => ({
        phone,
        index,
        result: undefined,
        error: error instanceof Error ? error.message : '查询失败',
        success: false,
        data: undefined,
      }))
    }
  }

  /**
   * 获取应用信息
   * @returns 应用信息
   */
  static async getAppInfo(): Promise<AppInfo | null> {
    try {
      const result = await invoke<AppInfo>('get_app_info')
      return result
    } catch (error) {
      console.error('获取应用信息失败:', error)
      return null
    }
  }

  /**
   * 清空缓存
   * @returns 是否成功
   */
  static async clearCache(): Promise<boolean> {
    try {
      await invoke('clear_cache')
      return true
    } catch (error) {
      console.error('清空缓存失败:', error)
      return false
    }
  }

  /**
   * 设置缓存大小
   * @param size 缓存大小
   * @returns 是否成功
   */
  static async setCacheSize(size: number): Promise<boolean> {
    try {
      await invoke('set_cache_size', { size })
      return true
    } catch (error) {
      console.error('设置缓存大小失败:', error)
      return false
    }
  }
}

// 工具函数
export const utils = {
  /**
   * 验证手机号格式
   * @param phone 手机号码
   * @returns 是否有效
   */
  isValidPhone(phone: string): boolean {
    // 中国大陆手机号正则：11位数字，以1开头
    const phoneRegex = /^1[3-9]\d{9}$/
    return phoneRegex.test(phone)
  },

  /**
   * 格式化手机号显示
   * @param phone 手机号码
   * @returns 格式化后的手机号
   */
  formatPhone(phone: string): string {
    if (phone.length !== 11) return phone
    return `${phone.slice(0, 3)} ${phone.slice(3, 7)} ${phone.slice(7)}`
  },

  /**
   * 复制文本到剪贴板
   * @param text 要复制的文本
   * @returns Promise<boolean> 是否成功
   */
  async copyToClipboard(text: string): Promise<boolean> {
    try {
      await navigator.clipboard.writeText(text)
      return true
    } catch (error) {
      console.error('复制失败:', error)
      // 降级方案：使用旧的复制API
      try {
        const textArea = document.createElement('textarea')
        textArea.value = text
        textArea.style.position = 'fixed'
        textArea.style.opacity = '0'
        document.body.appendChild(textArea)
        textArea.focus()
        textArea.select()
        const successful = document.execCommand('copy')
        document.body.removeChild(textArea)
        return successful
      } catch (fallbackError) {
        console.error('降级复制方案也失败:', fallbackError)
        return false
      }
    }
  },

  /**
   * 导出数据为CSV格式
   * @param data 要导出的数据
   * @param filename 文件名
   */
  exportToCSV(data: BatchQueryItem[], filename: string = 'phone-lookup-results.csv'): void {
    const headers = ['手机号', '省份', '城市', '邮编', '区号', '运营商类型', '状态']
    const csvContent = [
      headers.join(','),
      ...data.map(item => [
        item.phone,
        item.success ? item.data?.province || '' : '',
        item.success ? item.data?.city || '' : '',
        item.success ? item.data?.zip_code || '' : '',
        item.success ? item.data?.area_code || '' : '',
        item.success ? item.data?.card_type || '' : '',
        item.success ? '成功' : (item.error || '失败')
      ].map(field => `"${field}"`).join(','))
    ].join('\n')

    // 添加BOM以确保Excel正确显示中文
    const BOM = '\uFEFF'
    const blob = new Blob([BOM + csvContent], { type: 'text/csv;charset=utf-8' })
    
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = filename
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
  }
}