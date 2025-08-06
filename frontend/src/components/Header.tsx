import { useEffect } from 'react'
import { Smartphone, Info } from 'lucide-react'
import { useAppInfo } from '../hooks/usePhoneQuery'

export function Header() {
  const { appInfo, fetchAppInfo } = useAppInfo()

  useEffect(() => {
    fetchAppInfo()
  }, [fetchAppInfo])

  return (
    <header className="bg-white shadow-sm border-b border-gray-200">
      <div className="max-w-4xl mx-auto px-4 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-blue-600 rounded-lg">
              <Smartphone className="h-8 w-8 text-white" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900">手机号归属地查询工具</h1>
              <p className="text-sm text-gray-600">快速查询手机号码归属地信息</p>
            </div>
          </div>
          
          {appInfo && (
            <div className="flex items-center space-x-4 text-sm text-gray-600">
              <div className="flex items-center">
                <Info className="h-4 w-4 mr-1" />
                <span>数据库记录：{appInfo.database_info.total_records.toLocaleString()}</span>
              </div>
              <div className="hidden md:flex items-center">
                <span>缓存命中率：{appInfo.database_info.total_queries > 0 ? ((appInfo.database_info.cache_hits / appInfo.database_info.total_queries) * 100).toFixed(1) : '0.0'}%</span>
              </div>
              <div className="text-xs bg-gray-100 px-2 py-1 rounded">
                v{appInfo.version}
              </div>
            </div>
          )}
        </div>
      </div>
    </header>
  )
}