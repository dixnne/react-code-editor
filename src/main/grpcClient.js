import grpc from '@grpc/grpc-js';
import protoLoader from '@grpc/proto-loader';

const PROTO_PATH = '../protos/compilador.proto';
const packageDefinition = protoLoader.loadSync(PROTO_PATH);
const lexerProto = grpc.loadPackageDefinition(packageDefinition).lexer;

export async function runLexer(code, callback) {
  const client = new lexerProto.lexer('localhost:50051', grpc.credentials.createInsecure())
  client.Analyze({ input: code }, callback)
}
