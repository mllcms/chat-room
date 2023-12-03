import { moment } from '@/utils'

export interface User {
  id: string
  name: string
}
export interface Data {
  type: 'login' | 'logout' | 'system' | 'public' | 'private' | 'error'
  msg: string
  target: User
  list?: User[] | null
}

export function createData(type: Data['type'], msg: string, target: User): Data {
  return {
    type,
    msg,
    target,
  }
}

export interface SendMsg {
  type: 'public' | 'private'
  target: User
  msg: string
}

export interface MsgCache {
  position: 'left' | 'center' | 'right'
  target: User
  msg: string
  tiem: string
}

export function createMsgCache(position: MsgCache['position'], target: User, msg: string): MsgCache {
  return {
    position,
    target,
    msg,
    tiem: moment('MM月DD日 HH:mm:ss'),
  }
}
