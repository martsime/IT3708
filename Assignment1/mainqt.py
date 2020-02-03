import multiprocessing as mp
import sys
import time
from PyQt5.QtGui import QPainter, QColor, QBrush, QPen, QPainterPath, QFont
from PyQt5.QtCore import QThread, pyqtSignal, Qt, QRect
from PyQt5.QtWidgets import QApplication, QMainWindow, QWidget

from operator import itemgetter

import genetic

COLORS = [
    QColor('#ffa000'), # Orange
    QColor('#d32f2f'), # Red
    QColor('#388e3c'), # Green
    QColor('#6a1b9a'), # Purple
    QColor('#1976d2'), # Blue
    # QColor('#fbc02d'), # Yellow
    QColor('#00838f'), # Cyan
    QColor('#4e342e'), # Brown
]

class MessageType:
    BOUNDARIES = 'boundaries'
    CUSTOMERS = 'customers'
    DEPOTS = 'depots'
    ROUTES = 'routes'



def worker_process(q):
    program = genetic.GeneticProgram()
    q.put({
        MessageType.BOUNDARIES: program.get_boundaries(),
        MessageType.CUSTOMERS: program.get_customers(),
        MessageType.DEPOTS: program.get_depots(),
    })

    time.sleep(0.1)
    program.generate_population()
    old_solution = None
    while True:
        solution = program.simulate()
        if solution != old_solution:
            q.put({
                MessageType.ROUTES: solution,
            })
        old_solution = solution


class WorkerThread(QThread):
    boundaries_signal = pyqtSignal(object)
    customers_signal = pyqtSignal(object)
    depots_signal = pyqtSignal(object)
    routes_signal = pyqtSignal(object)

    def __init__(self, number=0):
        QThread.__init__(self)
        self.number = number

    def __del__(self):
        self.wait()

    def run(self):
        queue = mp.Queue()
        process = mp.Process(target=worker_process, args=[queue])
        process.start()

        while True:
            message = queue.get()
            if message:
                for key, value in message.items():
                    if key == MessageType.BOUNDARIES:
                        self.boundaries_signal.emit(value)
                    elif key == MessageType.CUSTOMERS:
                        self.customers_signal.emit(value)
                    elif key == MessageType.DEPOTS:
                        self.depots_signal.emit(value)
                    elif key == MessageType.ROUTES:
                        self.routes_signal.emit(value)
            time.sleep(0.01)
        return

class Window(QMainWindow):
    def __init__(self):
        super().__init__()
        self.size = (1000, 1000)
        self.setGeometry(0, 0, *self.size)

        self.boundaries = (0, 0, *self.size)
        self.raw_customers = {}
        self.raw_depots = {}
        self.customers = {}
        self.depots = {}
        self.routes = []

        self.start_worker()

    def init_painter():
        self.painter = QPainter()

    def start_worker(self):
        self.worker = WorkerThread()
        self.worker.boundaries_signal.connect(self.set_boundaries)
        self.worker.customers_signal.connect(self.update_customers)
        self.worker.depots_signal.connect(self.update_depots)
        self.worker.routes_signal.connect(self.update_routes)
        self.worker.start()

    def update_routes(self, routes):
        self.routes = routes
        self.update()

    def update_customers(self, customers):
        self.raw_customers = customers
        self.customers = self.transform_points(customers)
        self.update()

    def set_boundaries(self, boundaries):
        self.boundaries = boundaries
        self.update()

    def update_depots(self, depots):
        self.raw_depots = depots
        self.depots = self.transform_points(depots)
        self.update()

    def paintEvent(self, event):
        self.painter = QPainter()
        self.painter.begin(self)
        self.painter.setRenderHint(QPainter.Antialiasing)
        self.draw_routes()
        self.draw_depots()
        self.draw_customers()
        self.painter.end()

    def resizeEvent(self, event):
        self.customers = self.transform_points(self.raw_customers)
        self.depots = self.transform_points(self.raw_depots)
        self.update()


    def draw_depots(self):
        radius = 30
        for key, value in self.depots.items():
            color = COLORS[key % len(COLORS)]
            self.painter.setBrush(QBrush(color, Qt.SolidPattern))
            self.painter.setPen(Qt.NoPen)
            x, y = value
            new_x = int(x - radius/2)
            new_y = int(y - radius/2)
            ellipse = QRect(new_x, new_y, radius, radius)
            self.painter.drawEllipse(ellipse)
            self.painter.setPen(QPen(Qt.black))
            self.painter.drawText(ellipse, Qt.AlignCenter, f"{key}")

    def draw_customers(self):
        radius = 15
        for key, value in self.customers.items():
            self.painter.setBrush(QBrush(QColor('#bdbdbd'), Qt.SolidPattern))
            self.painter.setPen(Qt.NoPen)
            x, y = value
            new_x = int(x - radius/2)
            new_y = int(y - radius/2)
            ellipse = QRect(new_x, new_y, radius, radius)
            self.painter.drawEllipse(ellipse)
            self.painter.setFont(QFont('Times', 6))
            self.painter.setPen(QPen(Qt.black))
            self.painter.drawText(ellipse, Qt.AlignCenter, f"{key}")

    def draw_routes(self):
        positions = {**self.customers, **self.depots}
        for i, route in enumerate(self.routes):
            draw_path = QPainterPath()
            color = COLORS[i % len(COLORS)]
            self.painter.setPen(QPen(color, 2, Qt.SolidLine))
            start_index = route[0]
            route = route[1:]
            start_pos = positions.get(start_index)
            draw_path.moveTo(*start_pos)
            for index in route:
                pos = positions.get(index)
                draw_path.lineTo(*pos)
            self.painter.drawPath(draw_path)


    def transform_points(self, points, buffer=50):
        frame_geometry = self.frameGeometry()
        width = frame_geometry.width() - 2 * buffer
        height = frame_geometry.height() - 2 * buffer

        new_points = dict(points)
        min_x, min_y, max_x, max_y = self.boundaries

        # Translate to start from 0
        for key, val in new_points.items():
            x, y = val
            new_points[key] = (x - min_x, y - min_y)

        max_x -= min_x
        max_y -= min_y
        min_x = 0
        min_y = 0

        # Transform to fit screen
        for key, val in new_points.items():
            x, y = val

            # Flip y-axis to move origin to lower left
            y = max_y - y

            new_points[key] = (x / max_x * width + buffer, y / max_y * height + buffer)

        return new_points


if __name__ == '__main__':
    app = QApplication(sys.argv)
    window = Window()
    window.setAutoFillBackground(True)
    window.show()
    sys.exit(app.exec_())
