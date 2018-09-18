import { Observable } from 'rxjs';
/**
 * 将压缩包内的所有内容提取之指定目录，并保留原有文件结构
 * @param zipfile 压缩文件路径
 * @param dest 解压至目录
 */
export function extract(zipfile: string, dest: string): Observable<number>;
/**
 * 将文件目录中所有文件压缩成 zip 文件输出至指定位置
 * 原有的文件目录结构将被保留
 * @param src 需要压缩的文件目录
 * @param filepath 压缩文件的输出位置
 */
export function compress(src: string, filepath: string): Observable<number>;