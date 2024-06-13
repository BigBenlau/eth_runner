"""
The input of this script is the output of this command at server:
sudo ./transaction_test > {basename}.log
"""

import re
import sys
import csv

basename = sys.argv[1]
read_filename = "%s.log" % basename
read_file = open(read_filename)
output_filename = "%s_update.txt" % basename
output_file = open(output_filename, "w")

count_dict = {}
total_opcode_time = 0
total_opcode_time_except_call = 0


while True:
    line = read_file.readline()
    if not line:
        break
    if re.search("Opcode name is:", line) != None:
        opcode_line, time_line, count_line = line.split(" . ")
        opcode = opcode_line.replace("Opcode name is: ", "").replace(" ", "")
        total_dur_time = int(time_line.replace("Total Run time as nanos:", "").replace(" ", ""))
        total_count = int(count_line.replace("Total Count is:", "").replace(" ", ""))

        if count_dict.get(opcode, "Not") != "Not":
            raise ValueError
        else:
            count_dict[opcode] = {}

        count_dict[opcode]["count"] = total_count
        count_dict[opcode]["total_time"] = total_dur_time
        total_opcode_time += total_dur_time
        if opcode not in ["DELEGATECALL", "CALL", "STATICCALL"]:
            total_opcode_time_except_call += total_dur_time

print("scan file finished!!")
print("total opcode time used: %s" % round(total_opcode_time / 10**9, 4))
print("total opcode time except call: %s" % round(total_opcode_time_except_call / 10**9, 4))

for key in count_dict:
    count_dict_item = count_dict[key]
    count_dict[key]["avg_time"] = round(count_dict_item["total_time"] / count_dict_item["count"], 2)

sorted_list = sorted(count_dict.items(), key=lambda item:item[1]["avg_time"], reverse=True)

output_writer = csv.writer(output_file)
output_writer.writerow(["Opcode", "Average Time", "Count"])
for item in sorted_list:
    output_writer.writerow([item[0], item[1]["avg_time"], item[1]["count"]])