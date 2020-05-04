import os
import matplotlib.pyplot as plt
import pygenetic

GENERATIONS = int(os.getenv('GENERATIONS', '1000'))
BENCH_N = int(os.getenv('BENCH_N', '10'))



x = [i for i in range(1, GENERATIONS + 1)]

gen_5 = []
gen_10 = []
gen_20 = []
gen_50 = []
gen_100 = []
gen_200 = []
gen_500 = []

for var in [0.01, 0.02, 0.04, 0.08, 0.16, 0.32, 0.64, 1.0]:

    os.environ['SINGLE_SWAP_MUT_RATE'] = f'{var}'
    scores = [0 for i in range(0, GENERATIONS)]

    for i in range(0, BENCH_N):
        print(f"Benchmark {i + 1}")
        program = pygenetic.GeneticProgram()
        program.update_config()
        program.generate_population()
        for gen in range(0, GENERATIONS):
            score = program.get_solution_fitness()
            scores[gen] += score
            program.simulate()
        # print(scores)

    average_scores = [score / BENCH_N for score in scores]

    gen_5.append(average_scores[4])
    gen_10.append(average_scores[9])
    gen_20.append(average_scores[19])
    gen_50.append(average_scores[49])
    gen_100.append(average_scores[99])
    gen_200.append(average_scores[199])
    gen_500.append(average_scores[499])
    print(average_scores)
    plt.plot(x, average_scores, label=f'{var}')

print(f'Gen 5: {gen_5}')
print(f'Gen 10: {gen_10}')
print(f'Gen 20: {gen_20}')
print(f'Gen 50: {gen_50}')
print(f'Gen 100: {gen_100}')
print(f'Gen 200: {gen_200}')
print(f'Gen 500: {gen_500}')
plt.legend(title='Mutation rate')
plt.xlabel('Generations')
plt.ylabel('Fitness')
plt.title('Mutation rate settings')
plt.show()
