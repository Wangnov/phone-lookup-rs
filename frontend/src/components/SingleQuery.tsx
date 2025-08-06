import React, { useState } from 'react'
import { Search, Copy, Check, AlertCircle, Loader2 } from 'lucide-react'
import { usePhoneQuery, useCopyToClipboard } from '../hooks/usePhoneQuery'
import { utils } from '../lib/tauri'

export function SingleQuery() {
  const [phone, setPhone] = useState('')
  const { loading, result, error, query, reset } = usePhoneQuery()
  const { copied, copy } = useCopyToClipboard()

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    query(phone.trim())
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value.replace(/\D/g, '') // 只允许数字
    if (value.length <= 11) {
      setPhone(value)
      if (error) reset() // 清除错误状态
    }
  }

  const copyResult = async () => {
    if (result?.success && result.data) {
      const resultText = `手机号：${utils.formatPhone(phone)}
省份：${result.data.province}
城市：${result.data.city}
邮编：${result.data.zip_code}
区号：${result.data.area_code}
运营商：${result.data.card_type}`
      await copy(resultText)
    }
  }

  return (
    <div className="bg-white rounded-xl shadow-md border border-gray-100 overflow-hidden">
      <div className="p-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-6">单个号码查询</h2>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="phone" className="block text-sm font-medium text-gray-700 mb-2">
              手机号码
            </label>
            <div className="relative">
              <input
                id="phone"
                type="tel"
                value={phone}
                onChange={handleInputChange}
                placeholder="请输入11位手机号码"
                className={`px-4 py-3 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200 w-full ${error ? 'border-red-300 focus:ring-red-500' : 'border-gray-300'}`}
                disabled={loading}
                maxLength={11}
              />
              <Search className="absolute right-3 top-3 h-5 w-5 text-gray-400" />
            </div>
            {phone && phone.length > 0 && (
              <div className="mt-1 text-sm text-gray-500">
                格式化显示：{utils.formatPhone(phone)}
              </div>
            )}
            {error && (
              <div className="mt-2 flex items-center text-sm text-red-600">
                <AlertCircle className="h-4 w-4 mr-1" />
                {error}
              </div>
            )}
          </div>

          <button
            type="submit"
            disabled={!phone || phone.length !== 11 || loading}
            className="bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-6 rounded-lg transition-colors duration-200 disabled:opacity-50 disabled:cursor-not-allowed w-full flex items-center justify-center"
          >
            {loading ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                查询中...
              </>
            ) : (
              <>
                <Search className="h-4 w-4 mr-2" />
                查询归属地
              </>
            )}
          </button>
        </form>

        {/* 查询结果 */}
        {result && (
          <div className="mt-6 fade-in">
            {result.success && result.data ? (
              <div className="p-4 border-l-4 border-blue-600 bg-blue-50 rounded-r-lg">
                <div className="flex justify-between items-start mb-4">
                  <h3 className="text-lg font-semibold text-gray-900">查询结果</h3>
                  <button
                    onClick={copyResult}
                    className={`bg-gray-100 hover:bg-gray-200 text-gray-700 font-medium py-3 px-6 rounded-lg transition-colors duration-200 flex items-center text-sm ${copied ? 'bg-green-100 text-green-700' : ''
                      }`}
                    disabled={copied}
                  >
                    {copied ? (
                      <>
                        <Check className="h-4 w-4 mr-1" />
                        已复制
                      </>
                    ) : (
                      <>
                        <Copy className="h-4 w-4 mr-1" />
                        复制结果
                      </>
                    )}
                  </button>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="font-medium text-gray-600">手机号：</span>
                      <span className="font-mono">{utils.formatPhone(phone)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="font-medium text-gray-600">省份：</span>
                      <span>{result.data.province}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="font-medium text-gray-600">城市：</span>
                      <span>{result.data.city}</span>
                    </div>
                  </div>
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="font-medium text-gray-600">邮编：</span>
                      <span className="font-mono">{result.data.zip_code}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="font-medium text-gray-600">区号：</span>
                      <span className="font-mono">{result.data.area_code}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="font-medium text-gray-600">运营商：</span>
                      <span className="px-2 py-1 bg-blue-100 text-blue-800 rounded text-sm">
                        {result.data.card_type}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            ) : (
              <div className="p-4 border-l-4 border-red-400 bg-red-50 rounded-r-lg">
                <div className="flex items-center">
                  <AlertCircle className="h-5 w-5 text-red-400 mr-2" />
                  <span className="text-red-700">
                    {result.error || '查询失败，请检查手机号码是否正确'}
                  </span>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  )
}