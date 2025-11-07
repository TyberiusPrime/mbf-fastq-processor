with open("input_1.fq",'w') as op:
    op.write("@")
    op.write("A"* 1000)
    op.write("\n")
    op.write("AGTC" * int(1e6//4))
    op.write("\n")
    op.write("+\n")
    op.write("B"* int(1e6//1))
    op.write("\n")

