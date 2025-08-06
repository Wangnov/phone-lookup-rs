import { useState, useCallback } from 'react'
import { TauriAPI, type QueryResult, type BatchQueryResult, type BatchQueryItem, type AppInfo } from '../lib/tauri'
import { utils } from '../lib/tauri'

// 单个查询Hook
export function usePhoneQuery() {
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<QueryResult | null>(null)
  const [error, setError] = useState<string | null>(null)

  const query = useCallback(async (phone: string) => {
    if (!utils.isValidPhone(phone)) {
      setError('请输入有效的手机号码')
      return
    }

    setLoading(true)
    setError(null)
    
    try {
      const queryResult = await TauriAPI.queryPhone(phone)
      setResult(queryResult)
      
      if (!queryResult.success) {
        setError(queryResult.error || '查询失败')
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '查询失败'
      setError(errorMessage)
      setResult(null)
    } finally {
      setLoading(false)
    }
  }, [])

  const reset = useCallback(() => {
    setResult(null)
    setError(null)
    setLoading(false)
  }, [])

  return {
    loading,
    result,
    error,
    query,
    reset
  }
}

// 批量查询Hook
export function useBatchQuery() {
  const [loading, setLoading] = useState(false)
  const [results, setResults] = useState<BatchQueryResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [progress, setProgress] = useState(0)

  const query = useCallback(async (phones: string[]) => {
    // 验证输入
    const validPhones = phones.filter(phone => utils.isValidPhone(phone.trim()))
    const invalidPhones = phones.filter(phone => !utils.isValidPhone(phone.trim()))
    
    if (validPhones.length === 0) {
      setError('没有有效的手机号码')
      return
    }

    if (invalidPhones.length > 0) {
      console.warn('无效的手机号码:', invalidPhones)
    }

    setLoading(true)
    setError(null)
    setProgress(0)
    
    try {
      // 模拟进度更新
      const progressInterval = setInterval(() => {
        setProgress(prev => Math.min(prev + 10, 90))
      }, 100)

      const batchResults = await TauriAPI.queryPhonesBatch(validPhones)
      
      clearInterval(progressInterval)
      setProgress(100)
      
      // 处理结果，添加success字段和data字段以向后兼容
      const processedResults: BatchQueryItem[] = batchResults.map(item => ({
        ...item,
        success: !!item.result && !item.error,
        data: item.result, // 向后兼容
      }))
      
      // 为无效号码添加错误结果
      const invalidResults: BatchQueryItem[] = invalidPhones.map((phone, index) => ({
        phone: phone.trim(),
        index: validPhones.length + index,
        result: undefined,
        error: '无效的手机号码格式',
        success: false,
        data: undefined,
      }))

      const allResults = [...processedResults, ...invalidResults]
      const successful = allResults.filter(r => r.success).length
      const failed = allResults.filter(r => !r.success).length

      const finalResult: BatchQueryResult = {
        success: true,
        total: phones.length,
        successful,
        failed,
        results: allResults
      }

      setResults(finalResult)
      
      if (failed > 0) {
        setError(`部分查询失败，成功：${successful}，失败：${failed}`)
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '批量查询失败'
      setError(errorMessage)
      setResults(null)
    } finally {
      setLoading(false)
      setProgress(0)
    }
  }, [])

  const reset = useCallback(() => {
    setResults(null)
    setError(null)
    setLoading(false)
    setProgress(0)
  }, [])

  const exportCSV = useCallback((filename?: string) => {
    if (results && results.results.length > 0) {
      utils.exportToCSV(results.results, filename)
    }
  }, [results])

  return {
    loading,
    results,
    error,
    progress,
    query,
    reset,
    exportCSV
  }
}

// 复制功能Hook
export function useCopyToClipboard() {
  const [copied, setCopied] = useState(false)

  const copy = useCallback(async (text: string) => {
    const success = await utils.copyToClipboard(text)
    setCopied(success)
    
    if (success) {
      // 2秒后重置状态
      setTimeout(() => setCopied(false), 2000)
    }
    
    return success
  }, [])

  return {
    copied,
    copy
  }
}

// 应用信息Hook
export function useAppInfo() {
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null)
  const [loading, setLoading] = useState(false)

  const fetchAppInfo = useCallback(async () => {
    setLoading(true)
    try {
      const info = await TauriAPI.getAppInfo()
      setAppInfo(info)
    } catch (error) {
      console.error('获取应用信息失败:', error)
    } finally {
      setLoading(false)
    }
  }, [])

  return {
    appInfo,
    loading,
    fetchAppInfo
  }
}