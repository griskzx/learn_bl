# python 标准库
import argparse
import os
import struct
import sys
import zlib


def main():
    # 1.设置命令行参数解析
    parser = argparse.ArgumentParser(
        description="为固件bin文件计算并注入size环境和crc32"
    )
    parser.add_argument("bin_path", help="要处理的bin路径")
    args = parser.parse_args()
    bin_path = args.bin_path
    if not os.path.exists(bin_path):
        print(f"错误：找不到文件'{bin_path}''")
        sys.exit(1)
    with open(bin_path, "rb") as f:
        data = bytearray(f.read())

    if len(data) <= 256:
        print(f"错误：文件'{bin_path}'太小，小于256字节。")
        sys.exit(1)
    code_data = data[256:]
    actual_size = len(code_data)

    crc_value = zlib.crc32(code_data) & 0xFFFFFFFF

    struct.pack_into("<II", data, 8, actual_size, crc_value)

    with open(bin_path, "wb") as f:
        f.write(data)
        print(f"固件[{os.path.basename(bin_path)}] 打包成功")
        print(f"    ->真实代码大小:{actual_size} Bytes")
        print(f"    ->CRC32校验和:{hex(crc_value)}")


if __name__ == "__main__":
    main()
