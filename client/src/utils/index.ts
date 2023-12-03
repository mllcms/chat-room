import type { MsgCache } from '@/types'
import { reactive } from 'vue'

export function uuid(len?: number, radix?: number): string {
  var chars = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'.split('')
  var uuid = [],
    i
  radix = radix || chars.length
  if (len) {
    for (i = 0; i < len; i++) uuid[i] = chars[0 | (Math.random() * radix)]
  } else {
    var r
    uuid[8] = uuid[13] = uuid[18] = uuid[23] = '-'
    uuid[14] = '4'
    for (i = 0; i < 36; i++) {
      if (!uuid[i]) {
        r = 0 | (Math.random() * 16)
        uuid[i] = chars[i == 19 ? (r & 0x3) | 0x8 : r]
      }
    }
  }
  return uuid.join('')
}

/**
 * @param format YYYY-MM-DD HH:mm:ss
 * @param date new Date()
 * @returns string
 */
export function moment(format: string = 'YYYY-MM-DD HH:mm:ss', date = new Date()) {
  const obj: any = {
    YYYY: date.getFullYear(),
    YY: date.getFullYear().toString().slice(-2),
    MM: date.getMonth() + 1,
    DD: date.getDate(),
    HH: date.getHours().toString().padStart(2, '0'),
    mm: date.getMinutes().toString().padStart(2, '0'),
    ss: date.getSeconds().toString().padStart(2, '0'),
  }
  return format.replace(/Y{2,4}|MM|DD|HH|mm|ss/g, t => obj[t] || t)
}

export class CacheMsg {
  data: Record<string, { unread: number; data: MsgCache[] }>
  constructor() {
    this.data = reactive({})
  }
  get(name: string) {
    return (this.data[name] ||= { unread: 0, data: [] as MsgCache[] })
  }
  remove(name: string) {
    Reflect.deleteProperty(this.data, name)
  }
  getData(name: string) {
    return this.get(name).data
  }
  setData(name: string, data: MsgCache) {
    this.get(name).data.push(data)
  }

  getUnread(name: string) {
    return this.get(name).unread
  }

  setUnread(name: string, unread: number) {
    this.get(name).unread = unread
  }

  addUnread(name: string, num: number = 1) {
    this.get(name).unread += num
  }
}
