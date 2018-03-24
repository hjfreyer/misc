import tensorflow as tf
import pymongo as mgo
from bson.objectid import ObjectId

client = mgo.MongoClient()
db = client.nn2048
#from tensorflow.examples.tutorials.mnist import input_data

GRID_SIZE = 16
MAX_TILE = 15

#mnist = input_data.read_data_sets("/tmp/MNIST_data/", one_hot=True)
x = tf.placeholder(tf.float32, [None, GRID_SIZE * MAX_TILE])
W = tf.Variable(tf.zeros([GRID_SIZE * MAX_TILE, 1]))
b = tf.Variable(tf.zeros([1]))

y = tf.nn.softmax(tf.matmul(x, W) + b)
y_ = tf.placeholder(tf.float32, [None, 1])

cross_entropy = tf.reduce_mean(tf.nn.softmax_cross_entropy_with_logits(y, y_))
train_step = tf.train.GradientDescentOptimizer(0.5).minimize(cross_entropy)

init = tf.initialize_all_variables()

sess = tf.Session()
sess.run(init)

def get_game(state):
  game = db.games.find_one(state['gameid'])
  board = map(int, state['board'].split(','))
  boardvec = [0]*(GRID_SIZE*MAX_TILE)
  for i, p in enumerate(board):
    boardvec[i*MAX_TILE+p] = 1
  return boardvec, [game['score']]

states = db.states.find()

for i in range(1000):
  print 'data get'
  xs = []
  ys = []
  for i in range(100):
    bx, by = get_game(states.next())
    xs.append(bx)
    ys.append(by)

  print 'train step'
  sess.run(train_step, feed_dict={x: xs, y_: ys})

correct_prediction = tf.equal(tf.argmax(y,1), tf.argmax(y_,1))
accuracy = tf.reduce_mean(tf.cast(correct_prediction, tf.float32))
print(sess.run(accuracy, feed_dict={x: mnist.test.images, y_: mnist.test.labels}))
