import grpc from '@grpc/grpc-js';
import protoLoader from '@grpc/proto-loader';
import { join } from 'path'; // Es mejor usar join para rutas

// Definir rutas para AMBOS archivos proto
const LEXER_PROTO_PATH = join(__dirname, '../../protos/lexer.proto');
const PARSER_PROTO_PATH = join(__dirname, '../../protos/parser.proto');

// Opciones de carga del proto
const protoOptions = {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
  includeDirs: [join(__dirname, '../../protos')]
};

// Cargar AMBAS definiciones de proto
const packageDefinition = protoLoader.loadSync(
  [LEXER_PROTO_PATH, PARSER_PROTO_PATH],
  protoOptions
);

// Cargar la definición y acceder al paquete UNIFICADO 'compiler'
const compilerProto = grpc.loadPackageDefinition(packageDefinition).compiler;

// Crear un cliente para el servicio Lexer
const clientLexer = new compilerProto.Lexer('localhost:50051', grpc.credentials.createInsecure());
// (Si también usas el parser aquí, créalo de la misma forma)
// const clientParser = new compilerProto.Parser('localhost:50051', grpc.credentials.createInsecure());

// La función de exportación ahora usa el cliente correctamente configurado
export async function runLexer(code, callback) {
  clientLexer.Analyze({ input: code }, callback);
}