import os

# generate stress test for deposit
f = open(f"{os.getcwd()}/tests/resources/deposit_stress.csv", 'w')
f.write("type, client, tx, amount\n")
for i in range(0,1000000):
    f.write(f"deposit,1,{i},10.001\n")
    
f.close()