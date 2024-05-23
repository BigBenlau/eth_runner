import re

read_filename = "log_rust_runner.log"
read_file = open(read_filename)
output_filename = "opcode_average_time.txt"
output_file = open(output_filename, "w")

count_dict = {}

while True:
    line = read_file.readline()
    if not line:
        break
    if re.search("Opcode name is", line) != None:
        opcode_line, time_line = line.split(". ")
        opcode = opcode_line.replace("Opcode name is ", "")[1:-1]
        dur_time = int(time_line.replace("Run time as nanos: ", "")[:-1])

        if count_dict.get(opcode, "Not") == "Not":
            count_dict[opcode] = {
                "count": 0,
                "total_time": 0
            }

        count_dict[opcode]["count"] += 1
        count_dict[opcode]["total_time"] += dur_time

for key in count_dict:
    count_dict_item = count_dict[key]
    count_dict[key]["avg_time"] = round(count_dict_item["total_time"] / count_dict_item["count"], 2)

sorted_list = sorted(count_dict.items(), key=lambda item:item[1]["avg_time"], reverse=True)

for item in sorted_list:
    output_file.write("%s\t%s\n" % (item[0], item[1]["avg_time"]))