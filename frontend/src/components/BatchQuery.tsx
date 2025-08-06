import React, { useState } from 'react'
import { Upload, Download, Loader2, AlertCircle, CheckCircle, FileText, Trash2 } from 'lucide-react'
import { useBatchQuery } from '../hooks/usePhoneQuery'
import { utils } from '../lib/tauri'

export function BatchQuery() {
  const [phones, setPhones] = useState('')
  const { loading, results, error, progress, query, reset, exportCSV } = useBatchQuery()
  const [dragActive, setDragActive] = useState(false)

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    const phoneList = phones
      .split(/[\n,，\s]+/)
      .map(phone => phone.trim())
      .filter(phone => phone.length > 0)
    
    if (phoneList.length === 0) {
      return
    }

    query(phoneList)
  }

  const handleFileUpload = (file: File) => {
    const reader = new FileReader()
    reader.onload = (e) => {
      const content = e.target?.result as string
      if (content) {
        // 尝试解析CSV或纯文本
        let phoneList: string[] = []
        
        if (file.name.endsWith('.csv')) {
          // 简单的CSV解析
          const lines = content.split('\n')
          phoneList = lines.flatMap(line => 
            line.split(',').map(cell => cell.replace(/["\s]/g, ''))
          )
        } else {
          // 纯文本解析
          phoneList = content.split(/[\n,，\s]+/)
        }
        
        // 过滤有效的手机号码
        const validPhones = phoneList
          .map(phone => phone.trim())
          .filter(phone => phone.length > 0 && /^\d+$/.test(phone))
        
        setPhones(validPhones.join('\n'))
      }
    }
    reader.readAsText(file, 'UTF-8')
  }

  const handleDrag = (e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    if (e.type === 'dragenter' || e.type === 'dragover') {
      setDragActive(true)
    } else if (e.type === 'dragleave') {
      setDragActive(false)
    }
  }

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setDragActive(false)
    
    const files = e.dataTransfer.files
    if (files?.[0]) {
      handleFileUpload(files[0])
    }
  }

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      handleFileUpload(file)
    }
  }

  const clearPhones = () => {
    setPhones('')
    reset()
  }

  const phoneCount = phones
    .split(/[\n,，\s]+/)
    .map(phone => phone.trim())
    .filter(phone => phone.length > 0).length

  return (
    <div className="bg-white rounded-xl shadow-md border border-gray-100 overflow-hidden">
      <div className="p-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-6">批量号码查询</h2>
        
        <form onSubmit={handleSubmit} className="space-y-6">
          {/* 文件上传区域 */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              导入手机号码
            </label>
            <div
              className={`relative border-2 border-dashed rounded-lg p-6 text-center transition-colors
                ${dragActive 
                  ? 'border-primary bg-blue-50' 
                  : 'border-gray-300 hover:border-gray-400'
                }`}
              onDragEnter={handleDrag}
              onDragLeave={handleDrag}
              onDragOver={handleDrag}
              onDrop={handleDrop}
            >
              <input
                type="file"
                accept=".txt,.csv"
                onChange={handleFileSelect}
                className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
                disabled={loading}
              />
              <Upload className="mx-auto h-12 w-12 text-gray-400 mb-4" />
              <p className="text-gray-600">
                点击选择文件或拖拽文件到此处
              </p>
              <p className="text-sm text-gray-500 mt-2">
                支持 .txt 和 .csv 格式
              </p>
            </div>
          </div>

          {/* 文本输入区域 */}
          <div>
            <div className="flex justify-between items-center mb-2">
              <label htmlFor="phones" className="block text-sm font-medium text-gray-700">
                手机号码列表
              </label>
              {phones && (
                <button
                  type="button"
                  onClick={clearPhones}
                  className="text-sm text-gray-500 hover:text-red-600 flex items-center"
                >
                  <Trash2 className="h-4 w-4 mr-1" />
                  清空
                </button>
              )}
            </div>
            <textarea
              id="phones"
              value={phones}
              onChange={(e) => setPhones(e.target.value)}
              placeholder="请输入手机号码，每行一个或用逗号分隔&#10;例如：&#10;13812345678&#10;13987654321&#10;或：13812345678,13987654321"
              className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent transition-all duration-200 resize-none"
              rows={8}
              disabled={loading}
            />
            {phoneCount > 0 && (
              <div className="mt-2 text-sm text-gray-500">
                共计 {phoneCount} 个手机号码
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
            disabled={phoneCount === 0 || loading}
            className="bg-blue-600 hover:bg-blue-700 text-white font-medium py-3 px-6 rounded-lg transition-colors duration-200 disabled:opacity-50 disabled:cursor-not-allowed w-full flex items-center justify-center"
          >
            {loading ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                查询中... ({progress}%)
              </>
            ) : (
              <>
                <FileText className="h-4 w-4 mr-2" />
                批量查询 ({phoneCount} 个号码)
              </>
            )}
          </button>

          {/* 进度条 */}
          {loading && (
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div
                className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                style={{ width: `${progress}%` }}
              ></div>
            </div>
          )}
        </form>

        {/* 查询结果 */}
        {results && (
          <div className="mt-8 fade-in">
            {/* 统计信息 */}
            <div className="flex justify-between items-center mb-6">
              <div className="flex items-center space-x-6">
                <div className="flex items-center">
                  <CheckCircle className="h-5 w-5 text-green-500 mr-2" />
                  <span className="text-sm">
                    成功：<span className="font-medium text-green-600">{results.successful}</span>
                  </span>
                </div>
                <div className="flex items-center">
                  <AlertCircle className="h-5 w-5 text-red-500 mr-2" />
                  <span className="text-sm">
                    失败：<span className="font-medium text-red-600">{results.failed}</span>
                  </span>
                </div>
                <div className="text-sm text-gray-500">
                  总计：{results.total}
                </div>
              </div>
              
              <button
                onClick={() => exportCSV()}
                className="bg-gray-100 hover:bg-gray-200 text-gray-700 font-medium py-3 px-6 rounded-lg transition-colors duration-200 flex items-center"
              >
                <Download className="h-4 w-4 mr-2" />
                导出CSV
              </button>
            </div>

            {/* 结果列表 */}
            <div className="space-y-4 max-h-96 overflow-y-auto">
              {results.results.map((item, index) => (
                <div
                  key={index}
                  className={`p-4 rounded-lg border-l-4 ${
                    item.success
                      ? 'border-green-400 bg-green-50'
                      : 'border-red-400 bg-red-50'
                  }`}
                >
                  <div className="flex justify-between items-start">
                    <div className="flex-1">
                      <div className="flex items-center mb-2">
                        <span className="font-mono font-medium">
                          {utils.formatPhone(item.phone)}
                        </span>
                        {item.success ? (
                          <CheckCircle className="h-4 w-4 text-green-500 ml-2" />
                        ) : (
                          <AlertCircle className="h-4 w-4 text-red-500 ml-2" />
                        )}
                      </div>
                      
                      {item.success && item.data ? (
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-2 text-sm">
                          <div>
                            <span className="text-gray-600">省份：</span>
                            <span className="font-medium">{item.data.province}</span>
                          </div>
                          <div>
                            <span className="text-gray-600">城市：</span>
                            <span className="font-medium">{item.data.city}</span>
                          </div>
                          <div>
                            <span className="text-gray-600">区号：</span>
                            <span className="font-mono">{item.data.area_code}</span>
                          </div>
                          <div>
                            <span className="text-gray-600">运营商：</span>
                            <span className="px-2 py-1 bg-blue-100 text-blue-800 rounded text-xs">
                              {item.data.card_type}
                            </span>
                          </div>
                        </div>
                      ) : (
                        <div className="text-sm text-red-600">
                          {item.error || '查询失败'}
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}